use alloc::{collections::VecDeque, vec::Vec};
use core::num::{NonZeroU64, Wrapping};

use necsim_core::{
    cogs::{
        ActiveLineageSampler, DispersalSampler, Habitat, MinSpeciationTrackingEventSampler,
        PrimeableRng, SingularActiveLineageSampler, SpeciationProbability, TurnoverRate,
    },
    event::{DispersalEvent, LineageInteraction},
    landscape::IndexedLocation,
    lineage::{GlobalLineageReference, Lineage, MigratingLineage},
    reporter::{used::Unused, Reporter},
    simulation::Simulation,
};

use crate::{
    cogs::{
        active_lineage_sampler::independent::{
            event_time_sampler::EventTimeSampler, IndependentActiveLineageSampler,
        },
        coalescence_sampler::independent::IndependentCoalescenceSampler,
        emigration_exit::independent::{choice::EmigrationChoice, IndependentEmigrationExit},
        event_sampler::independent::IndependentEventSampler,
        immigration_entry::never::NeverImmigrationEntry,
        lineage_store::independent::IndependentLineageStore,
    },
    decomposition::Decomposition,
    partitioning::{LocalPartition, MigrationMode},
};

use super::{reporter::IgnoreProgressReporterProxy, DedupCache};

#[allow(clippy::type_complexity)]
pub fn simulate<
    H: Habitat,
    C: Decomposition<H>,
    E: EmigrationChoice<H>,
    G: PrimeableRng,
    D: DispersalSampler<H, G>,
    T: TurnoverRate<H>,
    N: SpeciationProbability<H>,
    J: EventTimeSampler<H, G, T>,
    R: Reporter,
    P: LocalPartition<R>,
>(
    mut simulation: Simulation<
        H,
        G,
        GlobalLineageReference,
        IndependentLineageStore<H>,
        IndependentEmigrationExit<H, C, E>,
        D,
        IndependentCoalescenceSampler<H>,
        T,
        N,
        IndependentEventSampler<H, G, IndependentEmigrationExit<H, C, E>, D, T, N>,
        NeverImmigrationEntry,
        IndependentActiveLineageSampler<H, G, IndependentEmigrationExit<H, C, E>, D, T, N, J>,
    >,
    mut lineages: VecDeque<Lineage>,
    dedup_cache: DedupCache,
    step_slice: NonZeroU64,
    local_partition: &mut P,
) -> (f64, u64) {
    let mut proxy = IgnoreProgressReporterProxy::from(local_partition);
    let mut min_spec_samples = dedup_cache.construct(lineages.len());

    // Ensure that the progress bar starts with the expected target
    proxy.local_partition().report_progress_sync(
        (Wrapping(lineages.len() as u64) + simulation.get_balanced_remaining_work()).0,
    );

    let mut immigration_events = Vec::new();

    let mut total_steps = 0_u64;
    let mut max_time = 0.0_f64;

    while !lineages.is_empty()
        || simulation.active_lineage_sampler().number_active_lineages() > 0
        || !simulation.emigration_exit().is_empty()
        || proxy.local_partition().wait_for_termination()
    {
        proxy.report_total_progress(
            (Wrapping(lineages.len() as u64) + simulation.get_balanced_remaining_work()).0,
        );

        let previous_task = simulation
            .active_lineage_sampler_mut()
            .replace_active_lineage(lineages.pop_front());

        let previous_speciation_sample =
            simulation.event_sampler_mut().replace_min_speciation(None);

        if let Some(previous_speciation_sample) = previous_speciation_sample {
            if min_spec_samples.insert(previous_speciation_sample) {
                if let Some(previous_task) = previous_task {
                    if previous_task.is_active() {
                        lineages.push_back(previous_task);
                    }
                }
            }
        }

        let (new_time, new_steps) =
            simulation.simulate_incremental_for(step_slice.get(), &mut proxy);

        total_steps += new_steps;
        max_time = max_time.max(new_time);

        // Force migration when no local tasks remain
        let migration_mode = if lineages.is_empty() {
            MigrationMode::Force
        } else {
            MigrationMode::Default
        };

        // Send off the possible emigrant and recieve immigrants
        let immigrants = match simulation.emigration_exit_mut().take() {
            Some(emigrant) => proxy.local_partition().migrate_individuals(
                &mut core::iter::once(emigrant),
                migration_mode,
                migration_mode,
            ),
            None => proxy.local_partition().migrate_individuals(
                &mut core::iter::empty(),
                migration_mode,
                migration_mode,
            ),
        };

        // Create local Lineages from the MigrantLineags
        for immigrant in immigrants {
            let MigratingLineage {
                global_reference,
                dispersal_origin,
                dispersal_target,
                prior_time,
                event_time,
                coalescence_rng_sample,
            } = immigrant;

            // Finish sampling the dispersal of the immigrating individual
            let target_index = coalescence_rng_sample.sample_coalescence_index(
                simulation
                    .habitat()
                    .get_habitat_at_location(&dispersal_target),
            );
            let dispersal_target = IndexedLocation::new(dispersal_target, target_index);

            // Cache the immigration event
            immigration_events.push(DispersalEvent {
                origin: dispersal_origin,
                prior_time,
                event_time,
                global_lineage_reference: global_reference.clone(),
                target: dispersal_target.clone(),
                interaction: LineageInteraction::Maybe,
            });

            // Append the new Lineage to the local task list
            lineages.push_back(Lineage::immigrate(
                global_reference,
                dispersal_target,
                event_time,
            ));
        }

        // Report any immigration events
        while let Some(immigration_event) = immigration_events.pop() {
            proxy.report_dispersal(Unused::new(&immigration_event));
        }
    }

    proxy.local_partition().report_progress_sync(0_u64);

    proxy
        .local_partition()
        .reduce_global_time_steps(max_time, total_steps)
}
