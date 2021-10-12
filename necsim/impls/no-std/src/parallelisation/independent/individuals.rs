use alloc::collections::VecDeque;
use core::{
    iter::FromIterator,
    num::{NonZeroU64, Wrapping},
};

use necsim_core_bond::NonNegativeF64;

use necsim_core::{
    cogs::{
        ActiveLineageSampler, DispersalSampler, Habitat, PrimeableRng, SpeciationProbability,
        TurnoverRate, F64Core
    },
    lineage::{GlobalLineageReference, Lineage},
    reporter::Reporter,
    simulation::Simulation,
};

use necsim_partitioning_core::LocalPartition;

use crate::cogs::{
    active_lineage_sampler::{
        independent::{event_time_sampler::EventTimeSampler, IndependentActiveLineageSampler},
        singular::SingularActiveLineageSampler,
    },
    coalescence_sampler::independent::IndependentCoalescenceSampler,
    emigration_exit::never::NeverEmigrationExit,
    event_sampler::{
        independent::IndependentEventSampler, tracking::MinSpeciationTrackingEventSampler,
    },
    immigration_entry::never::NeverImmigrationEntry,
    lineage_store::independent::IndependentLineageStore,
};

use super::{reporter::IgnoreProgressReporterProxy, DedupCache};

#[allow(clippy::type_complexity)]
pub fn simulate<
    F: F64Core,
    H: Habitat<F>,
    G: PrimeableRng<F>,
    D: DispersalSampler<F, H, G>,
    T: TurnoverRate<F, H>,
    N: SpeciationProbability<F, H>,
    J: EventTimeSampler<H, G, T>,
    R: Reporter,
    P: LocalPartition<R>,
    L: IntoIterator<Item = Lineage>,
>(
    mut simulation: Simulation<
        F,
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
    lineages: L,
    dedup_cache: DedupCache,
    step_slice: NonZeroU64,
    local_partition: &mut P,
) -> (NonNegativeF64, u64) {
    let mut lineages = VecDeque::from_iter(lineages);
    let mut proxy = IgnoreProgressReporterProxy::from(local_partition);
    let mut min_spec_samples = dedup_cache.construct(lineages.len());

    // Ensure that the progress bar starts with the expected target
    proxy.local_partition().report_progress_sync(
        (Wrapping(lineages.len() as u64) + simulation.get_balanced_remaining_work()).0,
    );

    let mut total_steps = 0_u64;
    let mut max_time = NonNegativeF64::zero();

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
                    lineages.push_back(previous_task);
                }
            }
        }

        let (new_time, new_steps) = simulation
            .simulate_incremental_early_stop(|_, steps, _| steps >= step_slice.get(), &mut proxy);

        total_steps += new_steps;
        max_time = max_time.max(new_time);
    }

    proxy.local_partition().report_progress_sync(0_u64);

    proxy
        .local_partition()
        .reduce_global_time_steps(max_time, total_steps)
}
