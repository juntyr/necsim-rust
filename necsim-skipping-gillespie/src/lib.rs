#![deny(clippy::pedantic)]

#[macro_use]
extern crate contracts;

use necsim_core::{
    cogs::{Habitat, LineageStore, RngCore, SeparableDispersalSampler},
    simulation::Simulation,
};

use necsim_impls_no_std::cogs::{
    coalescence_sampler::conditional::ConditionalCoalescenceSampler,
    event_sampler::gillespie::conditional::ConditionalGillespieEventSampler,
    lineage_reference::in_memory::InMemoryLineageReference,
    lineage_store::coherent::in_memory::CoherentInMemoryLineageStore,
};
use necsim_impls_std::cogs::{
    active_lineage_sampler::gillespie::GillespieActiveLineageSampler, rng::std::StdRng,
};

use necsim_impls_no_std::reporter::ReporterContext;

mod in_memory;
mod non_spatial;

pub struct SkippingGillespieSimulation;

impl SkippingGillespieSimulation {
    /// Simulates the Gillespie coalescence algorithm with self-dispersal event
    /// skipping on the `habitat` with `dispersal`.
    fn simulate<H: Habitat, D: SeparableDispersalSampler<H, StdRng>, P: ReporterContext>(
        habitat: H,
        dispersal_sampler: D,
        speciation_probability_per_generation: f64,
        sample_percentage: f64,
        seed: u64,
        reporter_context: P,
    ) -> (f64, u64) {
        reporter_context.with_reporter(|reporter| {
            let mut rng = StdRng::seed_from_u64(seed);
            let lineage_store = CoherentInMemoryLineageStore::new(sample_percentage, &habitat);
            let coalescence_sampler = ConditionalCoalescenceSampler::default();
            let event_sampler = ConditionalGillespieEventSampler::default();
            let active_lineage_sampler = GillespieActiveLineageSampler::new(
                speciation_probability_per_generation,
                &habitat,
                &dispersal_sampler,
                &lineage_store,
                &coalescence_sampler,
                &event_sampler,
                &mut rng,
            );

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
