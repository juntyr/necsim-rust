#![deny(clippy::pedantic)]
#![feature(never_type)]
#![allow(incomplete_features)]
#![feature(generic_associated_types)]

#[macro_use]
extern crate contracts;

use std::collections::VecDeque;

use lru::LruCache;

use necsim_core::{
    cogs::{
        DispersalSampler, Habitat, MinSpeciationTrackingEventSampler, RngCore,
        SingularActiveLineageSampler, SpeciationProbability, SpeciationSample,
    },
    lineage::{GlobalLineageReference, Lineage},
    reporter::Reporter,
    simulation::Simulation,
};

use necsim_impls_no_std::cogs::{
    active_lineage_sampler::independent::{
        event_time_sampler::exp::ExpEventTimeSampler, IndependentActiveLineageSampler,
    },
    coalescence_sampler::independent::IndependentCoalescenceSampler,
    emigration_exit::never::NeverEmigrationExit,
    event_sampler::independent::IndependentEventSampler,
    immigration_entry::never::NeverImmigrationEntry,
    lineage_store::independent::IndependentLineageStore,
    rng::seahash::SeaHash,
};

mod almost_infinite;
mod in_memory;
mod non_spatial;

mod reporter;
use reporter::DeduplicatingReporterProxy;

pub struct IndependentSimulation;

impl IndependentSimulation {
    /// Simulates the independent coalescence algorithm on the `habitat` with
    /// `dispersal` and lineages from `lineage_store`.
    fn simulate<
        H: Habitat,
        N: SpeciationProbability<H>,
        D: DispersalSampler<H, SeaHash>,
        R: Reporter,
    >(
        habitat: H,
        speciation_probability: N,
        dispersal_sampler: D,
        lineages: Vec<Lineage>,
        seed: u64,
        reporter: &mut R,
    ) -> (f64, u64) {
        const SIMULATION_STEP_SLICE: u64 = 10_u64;

        let mut reporter = DeduplicatingReporterProxy::from(reporter);

        let rng = SeaHash::seed_from_u64(seed);
        let lineage_store = IndependentLineageStore::default();
        let emigration_exit = NeverEmigrationExit::default();
        let coalescence_sampler = IndependentCoalescenceSampler::default();
        let event_sampler = IndependentEventSampler::default();
        let immigration_entry = NeverImmigrationEntry::default();
        let active_lineage_sampler = IndependentActiveLineageSampler::empty(
            ExpEventTimeSampler::new(1.0_f64), // FixedEventTimeSampler::default()
        );

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

        let mut min_spec_samples: LruCache<SpeciationSample, ()> =
            LruCache::new(lineages.len() * 5);

        let mut total_steps = 0_u64;
        let mut max_time = 0.0_f64;

        let mut lineages: VecDeque<Lineage> = lineages.into();

        while let Some(active_lineage) = lineages.pop_front() {
            reporter.report_total_progress(lineages.len() as u64);

            let previous_task = simulation
                .active_lineage_sampler_mut()
                .replace_active_lineage(Some(active_lineage));

            let previous_speciation_sample =
                simulation.event_sampler_mut().replace_min_speciation(None);

            if let Some(previous_speciation_sample) = previous_speciation_sample {
                if min_spec_samples
                    .put(previous_speciation_sample, ())
                    .is_none()
                {
                    if let Some(previous_task) = previous_task {
                        if previous_task.is_active() {
                            lineages.push_back(previous_task);
                        }
                    }
                }
            }

            let (new_time, new_steps) =
                simulation.simulate_incremental(SIMULATION_STEP_SLICE, &mut reporter);

            total_steps += new_steps;
            max_time = max_time.max(new_time);
        }

        (max_time, total_steps)
    }
}
