#![deny(clippy::pedantic)]
#![feature(never_type)]

#[macro_use]
extern crate contracts;

use necsim_core::{
    cogs::{CoherentLineageStore, DispersalSampler, Habitat, LineageReference, RngCore},
    simulation::Simulation,
};

use necsim_impls_no_std::cogs::{
    active_lineage_sampler::classical::ClassicalActiveLineageSampler,
    coalescence_sampler::unconditional::UnconditionalCoalescenceSampler,
    event_sampler::unconditional::UnconditionalEventSampler,
    speciation_probability::uniform::UniformSpeciationProbability,
};
use necsim_impls_std::cogs::rng::std::StdRng;

use necsim_impls_no_std::reporter::ReporterContext;

mod almost_infinite;
mod in_memory;
mod non_spatial;

pub struct ClassicalSimulation;

impl ClassicalSimulation {
    /// Simulates the classical coalescence algorithm on the `habitat` with
    /// `dispersal` and lineages from `lineage_store`.
    fn simulate<
        H: Habitat,
        D: DispersalSampler<H, StdRng>,
        R: LineageReference<H>,
        S: CoherentLineageStore<H, R>,
        P: ReporterContext,
    >(
        habitat: H,
        dispersal_sampler: D,
        lineage_store: S,
        speciation_probability_per_generation: f64,
        seed: u64,
        reporter_context: P,
    ) -> (f64, u64) {
        reporter_context.with_reporter(|reporter| {
            let rng = StdRng::seed_from_u64(seed);
            let speciation_probability =
                UniformSpeciationProbability::new(speciation_probability_per_generation);
            let coalescence_sampler = UnconditionalCoalescenceSampler::default();
            let event_sampler = UnconditionalEventSampler::default();
            let active_lineage_sampler = ClassicalActiveLineageSampler::new(&lineage_store);

            let simulation = Simulation::builder()
                .habitat(habitat)
                .rng(rng)
                .speciation_probability(speciation_probability)
                .dispersal_sampler(dispersal_sampler)
                .lineage_reference(std::marker::PhantomData::<R>)
                .lineage_store(lineage_store)
                .coalescence_sampler(coalescence_sampler)
                .event_sampler(event_sampler)
                .active_lineage_sampler(active_lineage_sampler)
                .build();

            let (time, steps) = simulation.simulate(reporter);

            (time, steps)
        })
    }
}
