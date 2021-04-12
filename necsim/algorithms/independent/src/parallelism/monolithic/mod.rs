use std::{
    collections::VecDeque,
    num::{NonZeroU32, NonZeroU64, Wrapping},
};

use necsim_core::{
    cogs::{
        ActiveLineageSampler, DispersalSampler, Habitat, MinSpeciationTrackingEventSampler,
        PrimeableRng, SingularActiveLineageSampler, SpeciationProbability, SpeciationSample,
        TurnoverRate,
    },
    event::{PackedEvent, TypedEvent},
    lineage::{GlobalLineageReference, Lineage},
    reporter::{used::Unused, Reporter},
    simulation::Simulation,
};

use necsim_impls_no_std::{
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

use crate::reporter::IgnoreProgressReporterProxy;

mod reporter;
use reporter::WaterLevelReporter;

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
    lineages: VecDeque<Lineage>,
    mut min_spec_samples: LruCache<SpeciationSample>,
    step_slice: NonZeroU64,
    event_slice: NonZeroU32,
    proxy: &mut IgnoreProgressReporterProxy<R, P>,
) -> (f64, u64) {
    let mut total_steps = 0_u64;
    let mut max_time = 0.0_f64;

    let mut slow_lineages = lineages;
    let mut fast_lineages = VecDeque::new();

    let mut slow_events: Vec<PackedEvent> = Vec::with_capacity(event_slice.get() as usize);
    let mut fast_events: Vec<PackedEvent> = Vec::with_capacity(event_slice.get() as usize);

    let mut level_time = 0.0_f64;

    while !slow_lineages.is_empty() {
        let total_event_rate: f64 = slow_lineages
            .iter()
            .map(|lineage| {
                simulation.turnover_rate().get_turnover_rate_at_location(
                    lineage.indexed_location().unwrap().location(),
                    simulation.habitat(),
                )
            })
            .sum();
        level_time += f64::from(event_slice.get()) / total_event_rate;

        slow_events.extend(fast_events.drain_filter(|event| event.time < level_time));

        let mut reporter: WaterLevelReporter<R> =
            WaterLevelReporter::new(level_time, &mut slow_events, &mut fast_events);

        while !slow_lineages.is_empty()
            || simulation.active_lineage_sampler().number_active_lineages() > 0
        {
            proxy.report_total_progress(
                (Wrapping(slow_lineages.len() as u64)
                    + Wrapping(fast_lineages.len() as u64)
                    + simulation.get_balanced_remaining_work())
                .0,
            );

            let previous_task = simulation
                .active_lineage_sampler_mut()
                .replace_active_lineage(slow_lineages.pop_front());

            let previous_speciation_sample =
                simulation.event_sampler_mut().replace_min_speciation(None);

            if let Some(previous_speciation_sample) = previous_speciation_sample {
                if min_spec_samples.insert(previous_speciation_sample) {
                    if let Some(previous_task) = previous_task {
                        if previous_task.is_active() {
                            if previous_task.last_event_time() < level_time {
                                slow_lineages.push_back(previous_task);
                            } else {
                                fast_lineages.push_back(previous_task);
                            }
                        }
                    }
                }
            }

            let (new_time, new_steps) =
                simulation.simulate_incremental_for(step_slice.get(), &mut reporter);

            total_steps += new_steps;
            max_time = max_time.max(new_time);
        }

        slow_events.sort();
        for event in slow_events.drain(..) {
            match event.into() {
                TypedEvent::Speciation(event) => {
                    proxy.report_speciation(Unused::new(&event));
                },
                TypedEvent::Dispersal(event) => {
                    proxy.report_dispersal(Unused::new(&event));
                },
            }
        }

        std::mem::swap(&mut slow_lineages, &mut fast_lineages);
    }

    (max_time, total_steps)
}
