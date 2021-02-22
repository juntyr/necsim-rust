use std::collections::VecDeque;

use anyhow::Result;
use lru_set::LruSet;

use necsim_core::{
    cogs::{
        ActiveLineageSampler, DispersalSampler, Habitat, MinSpeciationTrackingEventSampler,
        PrimeableRng, SingularActiveLineageSampler, SpeciationProbability, SpeciationSample,
    },
    event::{Event, EventType},
    landscape::IndexedLocation,
    lineage::{GlobalLineageReference, Lineage, MigratingLineage},
    reporter::Reporter,
    simulation::Simulation,
};

use necsim_impls_no_std::{
    cogs::{
        active_lineage_sampler::independent::{
            event_time_sampler::exp::ExpEventTimeSampler, IndependentActiveLineageSampler,
        },
        coalescence_sampler::independent::IndependentCoalescenceSampler,
        emigration_exit::independent::IndependentEmigrationExit,
        event_sampler::independent::IndependentEventSampler,
        immigration_entry::never::NeverImmigrationEntry,
        lineage_store::independent::IndependentLineageStore,
    },
    decomposition::Decomposition,
    partitioning::LocalPartition,
    reporter::ReporterContext,
};

use crate::{reporter::DeduplicatingReporterProxy, IndependentArguments};

#[allow(clippy::too_many_arguments)]
pub fn simulate<
    H: Habitat,
    C: Decomposition<H>,
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
    proxy: &mut DeduplicatingReporterProxy<R, P>,
    decomposition: C,
    mut min_spec_samples: LruSet<SpeciationSample>,
    auxiliary: &IndependentArguments,
) -> Result<(f64, u64)> {
    let step_slice = auxiliary.step_slice as u64;

    let emigration_exit = IndependentEmigrationExit::new(decomposition);
    let coalescence_sampler = IndependentCoalescenceSampler::default();
    let event_sampler = IndependentEventSampler::default();
    let immigration_entry = NeverImmigrationEntry::default();
    let active_lineage_sampler =
        IndependentActiveLineageSampler::empty(ExpEventTimeSampler::new(auxiliary.delta_t));

    let mut simulation = Simulation::builder()
        .habitat(habitat)
        .rng(rng)
        .speciation_probability(speciation_probability)
        .dispersal_sampler(dispersal_sampler)
        .lineage_reference(std::marker::PhantomData::<GlobalLineageReference>)
        .lineage_store(lineage_store)
        .emigration_exit(emigration_exit)
        .coalescence_sampler(coalescence_sampler)
        .event_sampler(event_sampler)
        .immigration_entry(immigration_entry)
        .active_lineage_sampler(active_lineage_sampler)
        .build();

    let mut immigration_events = Vec::new();

    let mut total_steps = 0_u64;
    let mut max_time = 0.0_f64;

    while !lineages.is_empty()
        || simulation.active_lineage_sampler().number_active_lineages() > 0
        || !simulation.emigration_exit().is_empty()
        || proxy.local_partition().wait_for_termination()
    {
        proxy.report_total_progress(
            (lineages.len() + simulation.active_lineage_sampler().number_active_lineages()) as u64,
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

        let (new_time, new_steps) = simulation.simulate_incremental(step_slice, proxy);

        total_steps += new_steps;
        max_time = max_time.max(new_time);

        // Send off the possible emigrant and recieve immigrants
        let immigrants = match simulation.emigration_exit_mut().take() {
            Some(emigrant) => proxy
                .local_partition()
                .migrate_individuals(&mut core::iter::once(emigrant)),
            None => proxy
                .local_partition()
                .migrate_individuals(&mut core::iter::empty()),
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
            immigration_events.push(Event::new(
                dispersal_origin,
                event_time,
                global_reference.clone(),
                EventType::Dispersal {
                    target: dispersal_target.clone(),
                    coalescence: None,
                },
            ));

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

    Ok((max_time, total_steps))
}