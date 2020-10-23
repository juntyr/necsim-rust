use anyhow::{Context, Result};
use array2d::Array2D;

use necsim_classical::ClassicalSimulation;
use necsim_corev2::reporter::Reporter;
use necsim_corev2::rng::Rng;
use necsim_gillespie::GillespieSimulation;
use necsim_implsv2::cogs::habitat::in_memory::InMemoryHabitat;
use necsim_implsv2::cogs::lineage_reference::in_memory::InMemoryLineageReference;
use necsim_skipping_gillespie::SkippingGillespieSimulation;

use super::args::{Algorithm, CommandLineArguments};

pub fn simulate(
    args: &CommandLineArguments,
    habitat: Array2D<u32>,
    dispersal: &Array2D<f64>,
    rng: &mut impl Rng,
    reporter: &mut impl Reporter<InMemoryHabitat, InMemoryLineageReference>,
) -> Result<(f64, usize)> {
    println!(
        "Setting up the {:?} coalescence algorithm ...",
        args.algorithm()
    );

    match args.algorithm() {
        Algorithm::Classical => ClassicalSimulation::simulate(
            habitat,
            &dispersal,
            *args.speciation_probability_per_generation(),
            *args.sample_percentage(),
            rng,
            reporter,
        ),
        Algorithm::Gillespie => GillespieSimulation::simulate(
            habitat,
            &dispersal,
            *args.speciation_probability_per_generation(),
            *args.sample_percentage(),
            rng,
            reporter,
        ),
        Algorithm::SkippingGillespie => SkippingGillespieSimulation::simulate(
            habitat,
            &dispersal,
            *args.speciation_probability_per_generation(),
            *args.sample_percentage(),
            rng,
            reporter,
        ),
    }
    .with_context(|| {
        format!(
            concat!(
                "Failed to create a Landscape with the habitat ",
                "map {:?} and the dispersal map {:?}."
            ),
            args.dispersal_map(),
            args.habitat_map()
        )
    })
}
