use std::{collections::VecDeque, num::Wrapping};

use necsim_core::{
    cogs::{
        ActiveLineageSampler, DispersalSampler, Habitat, MinSpeciationTrackingEventSampler,
        PrimeableRng, SingularActiveLineageSampler, SpeciationProbability, SpeciationSample,
    },
    event::DispersalEvent,
    landscape::IndexedLocation,
    lineage::{GlobalLineageReference, Lineage, MigratingLineage},
    reporter::Reporter,
    simulation::Simulation,
};

use necsim_impls_no_std::{
    cache::DirectMappedCache as LruCache,
    cogs::{
        active_lineage_sampler::independent::{
            event_time_sampler::exp::ExpEventTimeSampler, IndependentActiveLineageSampler,
        },
        coalescence_sampler::independent::IndependentCoalescenceSampler,
        emigration_exit::independent::{choice::EmigrationChoice, IndependentEmigrationExit},
        event_sampler::independent::IndependentEventSampler,
        immigration_entry::never::NeverImmigrationEntry,
        lineage_store::independent::IndependentLineageStore,
        turnover_rate::uniform::UniformTurnoverRate,
    },
    decomposition::Decomposition,
    partitioning::{LocalPartition, MigrationMode},
    reporter::ReporterContext,
};

use crate::{reporter::PartitionReporterProxy, IndependentArguments};

#[allow(clippy::too_many_arguments, clippy::too_many_lines)]
#[allow(clippy::needless_pass_by_value)]
pub fn simulate<
    H: Habitat,
    C: Decomposition<H>,
    E: EmigrationChoice<H>,
    G: PrimeableRng<H>,
    N: SpeciationProbability<H>,
    D: DispersalSampler<H, G>,
    R: ReporterContext,
    P: LocalPartition<R>,
>(
    habitat: H,
    rng: G,
    speciation_probability: N,
    dispersal_sampler: D,
    lineage_store: IndependentLineageStore<H>,
    mut lineages: VecDeque<Lineage>,
    proxy: &mut PartitionReporterProxy<R, P>,
    decomposition: C,
    emigration_choice: E,
    mut min_spec_samples: LruCache<SpeciationSample>,
    auxiliary: IndependentArguments,
) -> (f64, u64) {
    let step_slice = auxiliary.step_slice.get();

    let emigration_exit = IndependentEmigrationExit::new(decomposition, emigration_choice);
    let coalescence_sampler = IndependentCoalescenceSampler::default();
    let turnover_rate = UniformTurnoverRate::default();
    let event_sampler = IndependentEventSampler::default();
    let immigration_entry = NeverImmigrationEntry::default();
    let active_lineage_sampler =
        IndependentActiveLineageSampler::empty(ExpEventTimeSampler::new(auxiliary.delta_t.get()));

    let mut simulation = Simulation::builder()
        .habitat(habitat)
        .rng(rng)
        .speciation_probability(speciation_probability)
        .dispersal_sampler(dispersal_sampler)
        .lineage_reference(std::marker::PhantomData::<GlobalLineageReference>)
        .lineage_store(lineage_store)
        .emigration_exit(emigration_exit)
        .coalescence_sampler(coalescence_sampler)
        .turnover_rate(turnover_rate)
        .event_sampler(event_sampler)
        .immigration_entry(immigration_entry)
        .active_lineage_sampler(active_lineage_sampler)
        .build();

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

        let (new_time, new_steps) = simulation.simulate_incremental_for(step_slice, proxy);

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
                &mut std::iter::once(emigrant),
                migration_mode,
                migration_mode,
            ),
            None => proxy.local_partition().migrate_individuals(
                &mut std::iter::empty(),
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
            immigration_events.push(
                DispersalEvent {
                    origin: dispersal_origin,
                    time: event_time,
                    global_lineage_reference: global_reference.clone(),
                    target: dispersal_target.clone(),
                    coalescence: None,
                }
                .into(),
            );

            // Append the new Lineage to the local task list
            lineages.push_back(Lineage::immigrate(
                global_reference,
                dispersal_target,
                event_time,
            ));
        }

        // Report any immigration events
        while let Some(immigration_event) = immigration_events.pop() {
            proxy.report_event(&immigration_event);
        }
    }

    (max_time, total_steps)
}
