#![deny(clippy::pedantic)]
#![feature(never_type)]

#[macro_use]
extern crate contracts;

use necsim_core::{
    cogs::{DispersalSampler, Habitat, LineageStore, RngCore},
    simulation::Simulation,
};

use necsim_impls_no_std::cogs::{
    active_lineage_sampler::classical::ClassicalActiveLineageSampler,
    coalescence_sampler::unconditional::UnconditionalCoalescenceSampler,
    event_sampler::unconditional::UnconditionalEventSampler,
    lineage_reference::in_memory::InMemoryLineageReference,
    lineage_store::coherent::in_memory::CoherentInMemoryLineageStore,
};
use necsim_impls_std::cogs::rng::std::StdRng;

use necsim_impls_no_std::reporter::ReporterContext;

mod in_memory;
mod non_spatial;

pub struct ClassicalSimulation;

impl ClassicalSimulation {
    /// Simulates the classical coalescence algorithm on the `habitat` with
    /// `dispersal`.
    fn simulate<H: Habitat, D: DispersalSampler<H, StdRng>, P: ReporterContext>(
        habitat: H,
        dispersal_sampler: D,
        speciation_probability_per_generation: f64,
        sample_percentage: f64,
        seed: u64,
        reporter_context: P,
    ) -> (f64, u64) {
        reporter_context.with_reporter(|reporter| {
            let rng = StdRng::seed_from_u64(seed);
            let lineage_store = CoherentInMemoryLineageStore::new(sample_percentage, &habitat);
            let coalescence_sampler = UnconditionalCoalescenceSampler::default();
            let event_sampler = UnconditionalEventSampler::default();
            let active_lineage_sampler =
                ClassicalActiveLineageSampler::new(&habitat, &lineage_store);

            let simulation = Simulation::builder()
                .speciation_probability_per_generation(speciation_probability_per_generation)
                .habitat(habitat)
                .rng(rng)
                .dispersal_sampler(dispersal_sampler)
                .lineage_reference(std::marker::PhantomData::<InMemoryLineageReference>)
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
