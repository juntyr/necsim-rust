use anyhow::{Context, Result};
use array2d::Array2D;

#[cfg(feature = "necsim-classical")]
use necsim_classical::ClassicalSimulation;

#[cfg(feature = "necsim-cuda")]
use necsim_cuda::CudaSimulation;

#[cfg(feature = "necsim-gillespie")]
use necsim_gillespie::GillespieSimulation;

#[cfg(feature = "necsim-skipping-gillespie")]
use necsim_skipping_gillespie::SkippingGillespieSimulation;

use necsim_impls_no_std::cogs::{
    habitat::in_memory::InMemoryHabitat, lineage_reference::in_memory::InMemoryLineageReference,
};

#[allow(unused_imports)]
use super::args::{Algorithm, CommandLineArguments};

#[allow(unreachable_code)]
#[allow(unused_variables)]
#[allow(clippy::needless_pass_by_value)]
pub fn simulate(
    args: &CommandLineArguments,
    habitat: &Array2D<u32>,
    dispersal: &Array2D<f64>,
    rng: necsim_config::RngType! {},
    reporter: &mut necsim_config::ReporterType! {<InMemoryHabitat, InMemoryLineageReference>},
) -> Result<(f64, u64)> {
    println!(
        "Setting up the {:?} coalescence algorithm ...",
        args.algorithm()
    );

    #[allow(clippy::match_single_binding)]
    let result: Result<(f64, u64)> = match args.algorithm() {
        #[cfg(feature = "necsim-classical")]
        Algorithm::Classical => ClassicalSimulation::simulate(
            habitat,
            &dispersal,
            *args.speciation_probability_per_generation(),
            *args.sample_percentage(),
            rng,
            reporter,
        ),
        #[cfg(feature = "necsim-gillespie")]
        Algorithm::Gillespie => GillespieSimulation::simulate(
            habitat,
            &dispersal,
            *args.speciation_probability_per_generation(),
            *args.sample_percentage(),
            rng,
            reporter,
        ),
        #[cfg(feature = "necsim-skipping-gillespie")]
        Algorithm::SkippingGillespie => SkippingGillespieSimulation::simulate(
            habitat,
            &dispersal,
            *args.speciation_probability_per_generation(),
            *args.sample_percentage(),
            rng,
            reporter,
        ),
        #[cfg(feature = "necsim-cuda")]
        Algorithm::CUDA => CudaSimulation::simulate(
            habitat,
            &dispersal,
            *args.speciation_probability_per_generation(),
            *args.sample_percentage(),
            rng,
            reporter,
        ),
        #[allow(unreachable_patterns)]
        _ => anyhow::bail!("rustcoalescence does not support the selected algorithm"),
    };

    result.with_context(|| {
        format!(
            concat!(
                "Failed to create the Simulation with the habitat ",
                "map {:?} and the dispersal map {:?}."
            ),
            args.dispersal_map(),
            args.habitat_map()
        )
    })
}
