use anyhow::Result;

use necsim_algorithms::Algorithm;
use necsim_algorithms_monolithic::classical::ClassicalAlgorithm;
use necsim_core::cogs::{LineageStore, RngCore};
use necsim_impls_no_std::{
    cogs::origin_sampler::pre_sampler::OriginPreSampler, partitioning::LocalPartition,
    reporter::ReporterContext,
};
use necsim_scenarios::{
    almost_infinite::AlmostInfiniteScenario, non_spatial::NonSpatialScenario,
    spatially_explicit::SpatiallyExplicitScenario, spatially_implicit::SpatiallyImplicitScenario,
    Scenario,
};

use crate::args::{Algorithm as AlgorithmArgs, CommonArgs, Scenario as ScenarioArgs};

#[cfg(not(feature = "necsim-mpi"))]
pub mod monolithic;
#[cfg(feature = "necsim-mpi")]
pub mod mpi;

#[allow(clippy::module_name_repetitions)]
pub fn simulate_with_logger<R: ReporterContext, P: LocalPartition<R>>(
    mut local_partition: Box<P>,
    common_args: CommonArgs,
    scenario: ScenarioArgs,
) -> Result<()> {
    if local_partition.get_number_of_partitions().get() <= 1 {
        info!("The simulation will be run in monolithic mode.");
    } else {
        info!(
            "The simulation will be distributed across {} partitions.",
            local_partition.get_number_of_partitions().get()
        );
    }

    let pre_sampler = OriginPreSampler::all().percentage(common_args.sample_percentage.get());

    match common_args.algorithm {
        #[cfg(feature = "necsim-algorithms-monolithic")]
        AlgorithmArgs::Classical(algorithm_args) => {
            match scenario {
                ScenarioArgs::SpatiallyExplicit(scenario_args) => {
                    let scenario = SpatiallyExplicitScenario::initialise(
                        scenario_args,
                        common_args.speciation_probability_per_generation,
                    )?;

                    let result = ClassicalAlgorithm::initialise_and_simulate(
                        algorithm_args,
                        common_args.seed,
                        scenario,
                        pre_sampler,
                        &mut *local_partition,
                    );
                },
                // ScenarioArgs::NonSpatial(args) => simulate_with_scenario(
                //     local_partition,
                //     common_args,
                //     NonSpatialScenario::initialise(
                //         args,
                //         common_args.speciation_probability_per_generation,
                //     )
                //     .into_ok(),
                // ),
                // ScenarioArgs::AlmostInfinite(args) => simulate_with_scenario(
                //     local_partition,
                //     common_args,
                //     AlmostInfiniteScenario::initialise(
                //         args,
                //         common_args.speciation_probability_per_generation,
                //     )
                //     .into_ok(),
                // ),
                // ScenarioArgs::SpatiallyImplicit(args) => simulate_with_scenario(
                //     local_partition,
                //     common_args,
                //     SpatiallyImplicitScenario::initialise(
                //         args,
                //         common_args.speciation_probability_per_generation,
                //     )
                //     .into_ok(),
                // ),
                _ => return Ok(()),
            };
        },
        // #[cfg(feature = "necsim-algorithms-monolithic")]
        // Algorithm::Gillespie(..) => {},
        // #[cfg(feature = "necsim-algorithms-monolithic")]
        // Algorithm::SkippingGillespie(..) => {},
        // #[cfg(feature = "necsim-algorithms-independent")]
        // Algorithm::Independent(..) => {},
        // #[cfg(feature = "necsim-algorithms-cuda")]
        // Algorithm::Cuda(..) => {},
        _ => return Ok(()),
    };

    // match scenario {
    //     ScenarioArgs::SpatiallyExplicit(args) => simulate_with_scenario(
    //         local_partition,
    //         common_args,
    //         SpatiallyExplicitScenario::initialise(
    //             args,
    //             common_args.speciation_probability_per_generation,
    //         )?,
    //     ),
    //     ScenarioArgs::NonSpatial(args) => simulate_with_scenario(
    //         local_partition,
    //         common_args,
    //         NonSpatialScenario::initialise(args,
    // common_args.speciation_probability_per_generation)
    // .into_ok(),     ),
    //     ScenarioArgs::AlmostInfinite(args) => simulate_with_scenario(
    //         local_partition,
    //         common_args,
    //         AlmostInfiniteScenario::initialise(
    //             args,
    //             common_args.speciation_probability_per_generation,
    //         )
    //         .into_ok(),
    //     ),
    //     ScenarioArgs::SpatiallyImplicit(args) => simulate_with_scenario(
    //         local_partition,
    //         common_args,
    //         SpatiallyImplicitScenario::initialise(
    //             args,
    //             common_args.speciation_probability_per_generation,
    //         )
    //         .into_ok(),
    //     ),
    // };

    // let (time, steps) = match scenario {
    // Scenario::SpatiallyExplicit(spatially_explicit_args) => {
    // simulation::spatially_explicit::simulate(common_args,
    // spatially_explicit_args, local_partition.as_mut())? },
    // Scenario::NonSpatial(non_spatial_args) => simulation::non_spatial::simulate(
    // common_args,
    // non_spatial_args,
    // local_partition.as_mut(),
    // )?,
    // Scenario::SpatiallyImplicit(spatially_implicit_args) => {
    // simulation::spatially_implicit::simulate(
    // common_args,
    // spatially_implicit_args,
    // local_partition.as_mut(),
    // )?
    // },
    // Scenario::AlmostInfinite(almost_infinite_args) =>
    // simulation::almost_infinite::simulate( common_args,
    // almost_infinite_args,
    // local_partition.as_mut(),
    // )?,
    // };
    //
    // if log::log_enabled!(log::Level::Info) {
    // eprintln!("\n");
    // eprintln!("{:=^80}", " Reporter Summary ");
    // eprintln!();
    // }
    // local_partition.finalise_reporting();
    // if log::log_enabled!(log::Level::Info) {
    // eprintln!();
    // eprintln!("{:=^80}", " Reporter Summary ");
    // eprintln!();
    // }
    //
    // info!(
    // "The simulation finished at time {} after {} steps.\n",
    // time, steps
    // );

    Ok(())
}

// fn simulate_with_scenario<
//     G: RngCore,
//     L: LineageStore<S::Habitat, S::LineageReference>,
//     S: Scenario<G, L>,
//     R: ReporterContext,
//     P: LocalPartition<R>,
// >(
//     mut local_partition: Box<P>,
//     common_args: CommonArgs,
//     scenario: S,
// ) -> Result<()> {
//     Ok(())
// }
