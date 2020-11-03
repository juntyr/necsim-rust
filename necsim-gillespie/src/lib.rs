#![deny(clippy::pedantic)]

#[macro_use]
extern crate contracts;

use anyhow::Result;
use array2d::Array2D;

use necsim_core::cogs::LineageStore;
use necsim_core::reporter::Reporter;
use necsim_core::rng::Rng;
use necsim_core::simulation::Simulation;

use necsim_impls_no_std::cogs::coalescence_sampler::unconditional::UnconditionalCoalescenceSampler;
use necsim_impls_no_std::cogs::event_sampler::gillespie::unconditional::UnconditionalGillespieEventSampler;
use necsim_impls_no_std::cogs::habitat::in_memory::InMemoryHabitat;
use necsim_impls_no_std::cogs::lineage_reference::in_memory::InMemoryLineageReference;
use necsim_impls_std::cogs::active_lineage_sampler::gillespie::GillespieActiveLineageSampler;
use necsim_impls_std::cogs::dispersal_sampler::in_memory::alias::InMemoryAliasDispersalSampler;
use necsim_impls_std::cogs::dispersal_sampler::in_memory::InMemoryDispersalSampler;
use necsim_impls_std::cogs::lineage_store::coherent::in_memory::CoherentInMemoryLineageStore;

pub struct GillespieSimulation;

impl GillespieSimulation {
    /// Simulates the Gillespie coalescence algorithm on an in memory
    /// `habitat` with precalculated `dispersal`.
    ///
    /// # Errors
    ///
    /// `Err(InconsistentDispersalMapSize)` is returned iff the dimensions of
    /// `dispersal` are not `ExE` given `E=RxC` where `habitat` has dimension
    /// `RxC`.
    #[debug_requires(
        speciation_probability_per_generation >= 0.0_f64 &&
        speciation_probability_per_generation <= 1.0_f64,
        "0.0 <= speciation_probability_per_generation <= 1.0"
    )]
    #[debug_requires(
        sample_percentage >= 0.0_f64 &&
        sample_percentage <= 1.0_f64,
        "0.0 <= sample_percentage <= 1.0"
    )]
    pub fn simulate(
        habitat: &Array2D<u32>,
        dispersal: &Array2D<f64>,
        speciation_probability_per_generation: f64,
        sample_percentage: f64,
        rng: &mut impl Rng,
        reporter: &mut impl Reporter<InMemoryHabitat, InMemoryLineageReference>,
    ) -> Result<(f64, usize)> {
        let habitat = InMemoryHabitat::new(habitat.clone());
        let dispersal_sampler = InMemoryAliasDispersalSampler::new(dispersal, &habitat)?;
        let lineage_store = CoherentInMemoryLineageStore::new(sample_percentage, &habitat);
        let coalescence_sampler = UnconditionalCoalescenceSampler::default();
        let event_sampler = UnconditionalGillespieEventSampler::default();
        let active_lineage_sampler = GillespieActiveLineageSampler::new(
            speciation_probability_per_generation,
            &habitat,
            &dispersal_sampler,
            &lineage_store,
            &coalescence_sampler,
            &event_sampler,
            rng,
        );

        let simulation = Simulation::builder()
            .speciation_probability_per_generation(speciation_probability_per_generation)
            .habitat(habitat)
            .dispersal_sampler(dispersal_sampler)
            .lineage_reference(std::marker::PhantomData::<InMemoryLineageReference>)
            .lineage_store(lineage_store)
            .coalescence_sampler(coalescence_sampler)
            .event_sampler(event_sampler)
            .active_lineage_sampler(active_lineage_sampler)
            .build();

        let (time, steps) = simulation.simulate(rng, reporter);

        Ok((time, steps))
    }
}
