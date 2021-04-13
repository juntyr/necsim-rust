use alloc::collections::VecDeque;
use core::num::{NonZeroU64, Wrapping};

use necsim_core::{
    cogs::{
        ActiveLineageSampler, DispersalSampler, Habitat, MinSpeciationTrackingEventSampler,
        PrimeableRng, SingularActiveLineageSampler, SpeciationProbability, SpeciationSample,
        TurnoverRate,
    },
    lineage::{GlobalLineageReference, Lineage},
    simulation::Simulation,
};

use crate::{
    cache::DirectMappedCache as LruCache,
    cogs::{
        active_lineage_sampler::independent::{
            event_time_sampler::EventTimeSampler, IndependentActiveLineageSampler,
        },
        coalescence_sampler::independent::IndependentCoalescenceSampler,
        emigration_exit::never::NeverEmigrationExit,
        event_sampler::independent::IndependentEventSampler,
        immigration_entry::never::NeverImmigrationEntry,
        lineage_store::independent::IndependentLineageStore,
    },
    partitioning::LocalPartition,
    reporter::ReporterContext,
};

use super::reporter::IgnoreProgressReporterProxy;

#[allow(clippy::type_complexity)]
pub fn simulate<
    H: Habitat,
    G: PrimeableRng<H>,
    D: DispersalSampler<H, G>,
    T: TurnoverRate<H>,
    N: SpeciationProbability<H>,
    J: EventTimeSampler<H, G, T>,
    R: ReporterContext,
    P: LocalPartition<R>,
>(
    mut simulation: Simulation<
        H,
        G,
        GlobalLineageReference,
        IndependentLineageStore<H>,
        NeverEmigrationExit,
        D,
        IndependentCoalescenceSampler<H>,
        T,
        N,
        IndependentEventSampler<H, G, NeverEmigrationExit, D, T, N>,
        NeverImmigrationEntry,
        IndependentActiveLineageSampler<H, G, NeverEmigrationExit, D, T, N, J>,
    >,
    mut lineages: VecDeque<Lineage>,
    mut min_spec_samples: LruCache<SpeciationSample>,
    step_slice: NonZeroU64,
    local_partition: &mut P,
) -> (f64, u64) {
    let mut proxy = IgnoreProgressReporterProxy::from(local_partition);

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

        let (new_time, new_steps) =
            simulation.simulate_incremental_for(step_slice.get(), &mut proxy);

        total_steps += new_steps;
        max_time = max_time.max(new_time);
    }

    proxy.local_partition().report_progress_sync(0_u64);

    proxy
        .local_partition()
        .reduce_global_time_steps(max_time, total_steps)
}
