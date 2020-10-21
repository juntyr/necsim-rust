#![deny(clippy::pedantic)]

#[macro_use]
extern crate contracts;

use anyhow::Result;
use array2d::Array2D;

use necsim_core::reporter::Reporter;
use necsim_core::rng::Rng;
use necsim_core::{simulation::Simulation, simulation::SimulationSettings};
use necsim_impls::event_generator::lineage_sampler::active_list::ActiveLineageListSampler;
use necsim_impls::event_generator::unconditional_global::UnconditionalGlobalEventGenerator;
use necsim_impls::landscape::dispersal::in_memory::alias::InMemoryAliasDispersal as InMemoryDispersal;
use necsim_impls::landscape::in_memory_habitat_in_memory_dispersal::LandscapeInMemoryHabitatInMemoryDispersal;
//use necsim_impls::landscape::dispersal::in_memory::cumulative::InMemoryCumulativeDispersal as InMemoryDispersal;

pub struct ClassicalSimulation(std::marker::PhantomData<Simulation>);

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
        reporter: &mut impl Reporter,
    ) -> Result<(f64, usize)> {
        let landscape: LandscapeInMemoryHabitatInMemoryDispersal<InMemoryDispersal> =
            LandscapeInMemoryHabitatInMemoryDispersal::new(habitat, &dispersal)?;

        let settings = SimulationSettings::new(
            speciation_probability_per_generation,
            sample_percentage,
            landscape,
        );

        let (time, steps) = Simulation::simulate(
            &settings,
            UnconditionalGlobalEventGenerator::new(ActiveLineageListSampler::new(&settings)),
            rng,
            reporter,
        );

        Ok((time, steps))
    }
}
