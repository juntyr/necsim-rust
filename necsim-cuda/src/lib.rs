#![deny(clippy::pedantic)]

#[macro_use]
extern crate contracts;

use anyhow::Result;
use array2d::Array2D;

use necsim_core::cogs::LineageStore;
use necsim_core::reporter::Reporter;
use necsim_core::rng::Rng;
use necsim_core::simulation::Simulation;

use necsim_impls_no_std::cogs::active_lineage_sampler::independent::IndependentActiveLineageSampler;
use necsim_impls_no_std::cogs::coalescence_sampler::independent::IndependentCoalescenceSampler;
use necsim_impls_no_std::cogs::dispersal_sampler::in_memory::packed_alias::InMemoryPackedAliasDispersalSampler;
use necsim_impls_no_std::cogs::event_sampler::independent::IndependentEventSampler;
use necsim_impls_no_std::cogs::habitat::in_memory::InMemoryHabitat;
use necsim_impls_no_std::cogs::lineage_reference::in_memory::InMemoryLineageReference;
use necsim_impls_no_std::cogs::lineage_store::incoherent::in_memory::IncoherentInMemoryLineageStore;
use necsim_impls_std::cogs::dispersal_sampler::in_memory::InMemoryDispersalSampler;

pub struct CudaSimulation;

impl CudaSimulation {
    /// Simulates the coalescence algorithm on a CUDA-capable GPU on an in
    /// memory `habitat` with precalculated `dispersal`.
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
        let dispersal_sampler = InMemoryPackedAliasDispersalSampler::new(dispersal, &habitat)?;
        let lineage_store = IncoherentInMemoryLineageStore::new(sample_percentage, &habitat);
        let coalescence_sampler = IndependentCoalescenceSampler::default();
        let event_sampler = IndependentEventSampler::default();
        let active_lineage_sampler = IndependentActiveLineageSampler::new(
            InMemoryLineageReference::from(0_usize),
            &lineage_store,
        ); // TODO

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
