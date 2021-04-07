use std::{collections::VecDeque, num::Wrapping};

use necsim_core::{
    cogs::{
        ActiveLineageSampler, DispersalSampler, Habitat, MinSpeciationTrackingEventSampler,
        PrimeableRng, SingularActiveLineageSampler, SpeciationProbability, SpeciationSample,
    },
    lineage::{GlobalLineageReference, Lineage},
    simulation::Simulation,
};

use necsim_impls_no_std::{
    cache::DirectMappedCache as LruCache,
    cogs::{
        active_lineage_sampler::independent::{
            event_time_sampler::exp::ExpEventTimeSampler, IndependentActiveLineageSampler,
        },
        coalescence_sampler::independent::IndependentCoalescenceSampler,
        emigration_exit::never::NeverEmigrationExit,
        event_sampler::independent::IndependentEventSampler,
        immigration_entry::never::NeverImmigrationEntry,
        lineage_store::independent::IndependentLineageStore,
        turnover_rate::uniform::UniformTurnoverRate,
    },
    partitioning::LocalPartition,
    reporter::ReporterContext,
};

use crate::{reporter::PartitionReporterProxy, IndependentArguments};

#[allow(clippy::too_many_arguments)]
#[allow(clippy::needless_pass_by_value)]
pub fn simulate<
    H: Habitat,
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
    mut min_spec_samples: LruCache<SpeciationSample>,
    auxiliary: IndependentArguments,
) -> (f64, u64) {
    let step_slice = auxiliary.step_slice.get();

    let emigration_exit = NeverEmigrationExit::default();
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

    let mut total_steps = 0_u64;
    let mut max_time = 0.0_f64;

    while !lineages.is_empty()
        || simulation.active_lineage_sampler().number_active_lineages() > 0
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
    }

    (max_time, total_steps)
}
