use std::{collections::VecDeque, num::Wrapping};

use necsim_core::{
    cogs::{
        ActiveLineageSampler, DispersalSampler, Habitat, MinSpeciationTrackingEventSampler,
        PrimeableRng, SingularActiveLineageSampler, SpeciationProbability, SpeciationSample,
        TurnoverRate,
    },
    event::Event,
    lineage::{GlobalLineageReference, Lineage},
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

mod reporter;
use reporter::WaterLevelReporter;

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
    lineages: VecDeque<Lineage>,
    proxy: &mut PartitionReporterProxy<R, P>,
    mut min_spec_samples: LruCache<SpeciationSample>,
    auxiliary: IndependentArguments,
) -> (f64, u64) {
    const EVENT_BUFFER_SIZE: u32 = 1_000_000_u32;

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

    let mut total_steps = 0_u64;
    let mut max_time = 0.0_f64;

    let mut slow_lineages = lineages;
    let mut fast_lineages = VecDeque::new();

    let mut slow_events: Vec<Event> = Vec::with_capacity(EVENT_BUFFER_SIZE as usize);
    let mut fast_events: Vec<Event> = Vec::with_capacity(EVENT_BUFFER_SIZE as usize);

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
        level_time += f64::from(EVENT_BUFFER_SIZE) / total_event_rate;

        slow_events.extend(fast_events.drain_filter(|event| event.time() < level_time));

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
                simulation.simulate_incremental_for(step_slice, &mut reporter);

            total_steps += new_steps;
            max_time = max_time.max(new_time);
        }

        slow_events.sort();
        for event in &slow_events {
            proxy.report_event(event);
        }
        slow_events.clear();

        std::mem::swap(&mut slow_lineages, &mut fast_lineages);
    }

    (max_time, total_steps)
}
