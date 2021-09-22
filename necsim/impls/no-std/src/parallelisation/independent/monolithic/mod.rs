use alloc::collections::VecDeque;
use core::num::{NonZeroU64, Wrapping};

use necsim_core_bond::{NonNegativeF64, PositiveF64};

use necsim_core::{
    cogs::{
        ActiveLineageSampler, DispersalSampler, Habitat, PrimeableRng, SpeciationProbability,
        TurnoverRate,
    },
    lineage::{GlobalLineageReference, Lineage},
    reporter::{boolean::Boolean, Reporter},
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

use crate::parallelisation::independent::{DedupCache, EventSlice};

pub mod reporter;

use reporter::{
    WaterLevelReporterConstructor, WaterLevelReporterProxy, WaterLevelReporterStrategy,
};

#[allow(clippy::type_complexity, clippy::too_many_lines)]
pub fn simulate<
    H: Habitat,
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
        NeverEmigrationExit,
        D,
        IndependentCoalescenceSampler<H>,
        T,
        N,
        IndependentEventSampler<H, G, NeverEmigrationExit, D, T, N>,
        NeverImmigrationEntry,
        IndependentActiveLineageSampler<H, G, NeverEmigrationExit, D, T, N, J>,
    >,
    lineages: VecDeque<(Lineage, NonNegativeF64)>,
    dedup_cache: DedupCache,
    step_slice: NonZeroU64,
    event_slice: EventSlice,
    local_partition: &mut P,
) -> (NonNegativeF64, u64) {
    // Ensure that the progress bar starts with the expected target
    local_partition.report_progress_sync(
        (Wrapping(lineages.len() as u64) + simulation.get_balanced_remaining_work()).0,
    );

    let event_slice = event_slice.capacity(lineages.len());

    let mut proxy = <WaterLevelReporterStrategy as WaterLevelReporterConstructor<
        P::IsLive,
        R,
        P,
    >>::WaterLevelReporter::new(event_slice.get(), local_partition);
    let mut min_spec_samples = dedup_cache.construct(lineages.len());

    let mut total_steps = 0_u64;
    let mut max_time = NonNegativeF64::zero();

    let mut slow_lineages = lineages;
    let mut fast_lineages = VecDeque::new();

    let mut level_time = NonNegativeF64::zero();

    while !slow_lineages.is_empty() {
        // Calculate a new water-level time which all individuals should reach
        let total_event_rate: NonNegativeF64 = if R::ReportDispersal::VALUE {
            // Full event rate lambda with speciation
            slow_lineages
                .iter()
                .map(|(lineage, _)| {
                    simulation.turnover_rate().get_turnover_rate_at_location(
                        lineage.indexed_location.location(),
                        simulation.habitat(),
                    )
                })
                .sum()
        } else if R::ReportSpeciation::VALUE {
            // Only speciation event rate lambda * nu
            slow_lineages
                .iter()
                .map(|(lineage, _)| {
                    let location = lineage.indexed_location.location();

                    simulation
                        .turnover_rate()
                        .get_turnover_rate_at_location(location, simulation.habitat())
                        * simulation
                            .speciation_probability()
                            .get_speciation_probability_at_location(location, simulation.habitat())
                })
                .sum()
        } else {
            // No events produced -> no restriction
            NonNegativeF64::zero()
        };

        level_time += NonNegativeF64::from(event_slice.get()) / total_event_rate;

        // [Report all events below the water level] + Advance the water level
        proxy.advance_water_level(level_time);

        let mut previous_next_event_time: Option<PositiveF64> = None;

        // Simulate all slow lineages until they have finished or exceeded the new water
        //  level
        while !slow_lineages.is_empty()
            || simulation.active_lineage_sampler().number_active_lineages() > 0
        {
            let next_slow_lineage = loop {
                match slow_lineages.pop_front() {
                    None => break None,
                    Some((slow_lineage, next_event)) if next_event < level_time => {
                        break Some(slow_lineage)
                    },
                    Some((fast_lineage, next_event)) => {
                        fast_lineages.push_back((fast_lineage, next_event));
                    },
                }
            };

            let previous_task = simulation
                .active_lineage_sampler_mut()
                .replace_active_lineage(next_slow_lineage);

            let previous_speciation_sample =
                simulation.event_sampler_mut().replace_min_speciation(None);

            if let (Some(previous_task), Some(previous_next_event_time)) =
                (previous_task, previous_next_event_time)
            {
                let duplicate_individual = previous_speciation_sample
                    .map_or(false, |spec_sample| !min_spec_samples.insert(spec_sample));

                if !duplicate_individual {
                    // Reclassify lineages as either slow (still below water) or fast
                    if previous_next_event_time < level_time {
                        slow_lineages.push_back((previous_task, previous_next_event_time.into()));
                    } else {
                        fast_lineages.push_back((previous_task, previous_next_event_time.into()));
                    }
                }
            }

            previous_next_event_time = None;

            let (new_time, new_steps) = simulation.simulate_incremental_early_stop(
                |_, steps, next_event_time| {
                    previous_next_event_time = Some(next_event_time);

                    steps >= step_slice.get() || next_event_time >= level_time
                },
                &mut proxy,
            );

            total_steps += new_steps;
            max_time = max_time.max(new_time);

            proxy.local_partition().get_reporter().report_progress(
                &(Wrapping(slow_lineages.len() as u64)
                    + Wrapping(fast_lineages.len() as u64)
                    + simulation.get_balanced_remaining_work())
                .0
                .into(),
            );
        }

        // Fast lineages are now slow again
        core::mem::swap(&mut slow_lineages, &mut fast_lineages);
    }

    // [Report all remaining events]
    proxy.finalise();

    local_partition.report_progress_sync(0_u64);

    local_partition.reduce_global_time_steps(max_time, total_steps)
}
