#![deny(clippy::pedantic)]

#[macro_use]
extern crate contracts;

use anyhow::Result;
use array2d::Array2D;

use necsim_corev2::cogs::LineageStore;
use necsim_corev2::reporter::Reporter;
use necsim_corev2::rng::Rng;
use necsim_corev2::simulation::Simulation;

use necsim_implsv2::cogs::active_lineage_sampler::classical::ClassicalActiveLineageSampler;
use necsim_implsv2::cogs::coalescence_sampler::unconditional::UnconditionalCoalescenceSampler;
use necsim_implsv2::cogs::dispersal_sampler::in_memory::alias::InMemoryAliasDispersalSampler;
use necsim_implsv2::cogs::dispersal_sampler::in_memory::InMemoryDispersalSampler;
use necsim_implsv2::cogs::event_sampler::unconditional::UnconditionalEventSampler;
use necsim_implsv2::cogs::habitat::in_memory::InMemoryHabitat;
use necsim_implsv2::cogs::lineage_reference::in_memory::InMemoryLineageReference;
use necsim_implsv2::cogs::lineage_store::in_memory::InMemoryLineageStore;

pub struct ClassicalSimulation;

impl ClassicalSimulation {
    /// Simulates the classical coalescence algorithm on an in memory
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
        habitat: Array2D<u32>,
        dispersal: &Array2D<f64>,
        speciation_probability_per_generation: f64,
        sample_percentage: f64,
        rng: &mut impl Rng,
        reporter: &mut impl Reporter<InMemoryHabitat, InMemoryLineageReference>,
    ) -> Result<(f64, usize)> {
        let habitat = InMemoryHabitat::new(habitat);
        let dispersal_sampler = InMemoryAliasDispersalSampler::new(dispersal, &habitat)?;
        let lineage_store = InMemoryLineageStore::new(sample_percentage, &habitat);
        let coalescence_sampler = UnconditionalCoalescenceSampler;
        let event_sampler = UnconditionalEventSampler;
        let active_lineage_sampler = ClassicalActiveLineageSampler::new(&habitat, &lineage_store);

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
