use anyhow::Context;
use log::LevelFilter;

use necsim_partitioning_core::Partitioning as _;
use necsim_plugins_core::match_any_reporter_plugin_vec;

use crate::{
    args::{CommandArgs, Partitioning, SimulateArgs},
    reporter::DynamicReporterContext,
};

#[macro_use]
mod r#impl;

#[cfg(any(
    feature = "rustcoalescence-algorithms-gillespie",
    feature = "rustcoalescence-algorithms-independent",
    feature = "rustcoalescence-algorithms-cuda"
))]
mod dispatch;

#[allow(clippy::module_name_repetitions)]
pub fn simulate_with_logger(simulate_args: CommandArgs) -> anyhow::Result<()> {
    log::set_max_level(LevelFilter::Info);

    let simulate_args = SimulateArgs::try_parse(simulate_args)?;

    let simulate_args_info = format!("{:#?}", simulate_args);
    let post_validation = move || {
        info!("Parsed simulation arguments:\n{}", simulate_args_info);
    };

    let event_log_directory = simulate_args
        .event_log
        .as_ref()
        .map(|event_log| format!("{:?}", event_log));
    let pre_launch = move || {
        if let Some(event_log_directory) = event_log_directory {
            info!(
                "The simulation will log its events to {}.",
                event_log_directory
            );
            warn!("Therefore, only progress will be reported live.");
        } else {
            info!("The simulation will report events live.");
        }
    };

    match_any_reporter_plugin_vec!(simulate_args.reporters => |reporter| {
        use necsim_partitioning_monolithic::MonolithicLocalPartition;
        #[cfg(feature = "necsim-partitioning-mpi")]
        use necsim_partitioning_mpi::MpiLocalPartition;

        // Initialise the local partition and the simulation
        match simulate_args.partitioning {
            Partitioning::Monolithic(partitioning) => match partitioning.into_local_partition(
                DynamicReporterContext::new(reporter), simulate_args.event_log
            ).with_context(|| "Failed to initialise the local monolithic partition.")? {
                MonolithicLocalPartition::Live(partition) => dispatch::simulate_with_logger(
                    partition, simulate_args.common, simulate_args.scenario,
                    simulate_args.algorithm, simulate_args.pause,
                    post_validation, pre_launch,
                ),
                MonolithicLocalPartition::Recorded(partition) => dispatch::simulate_with_logger(
                    partition, simulate_args.common, simulate_args.scenario,
                    simulate_args.algorithm, simulate_args.pause,
                    post_validation, pre_launch,
                ),
            },
            #[cfg(feature = "necsim-partitioning-mpi")]
            Partitioning::Mpi(partitioning) => match partitioning.into_local_partition(
                DynamicReporterContext::new(reporter), simulate_args.event_log
            ).with_context(|| "Failed to initialise the local MPI partition.")? {
                MpiLocalPartition::Root(partition) => dispatch::simulate_with_logger(
                    partition, simulate_args.common, simulate_args.scenario,
                    simulate_args.algorithm, simulate_args.pause,
                    post_validation, pre_launch,
                ),
                MpiLocalPartition::Parallel(partition) => dispatch::simulate_with_logger(
                    partition, simulate_args.common, simulate_args.scenario,
                    simulate_args.algorithm, simulate_args.pause,
                    post_validation, pre_launch,
                ),
            },
        }
    })
}

#[cfg(not(any(
    feature = "rustcoalescence-algorithms-gillespie",
    feature = "rustcoalescence-algorithms-independent",
    feature = "rustcoalescence-algorithms-cuda"
)))]
mod dispatch {
    use necsim_core::reporter::Reporter;
    use necsim_partitioning_core::LocalPartition;

    use crate::args::{Algorithm as AlgorithmArgs, CommonArgs, Scenario as ScenarioArgs};

    #[allow(clippy::boxed_local, clippy::needless_pass_by_value)]
    pub fn simulate_with_logger<R: Reporter, P: LocalPartition<R>, V: FnOnce(), L: FnOnce()>(
        _local_partition: Box<P>,
        _common_args: CommonArgs,
        _scenario: ScenarioArgs,
        _algorithm: AlgorithmArgs,
        _post_validation: V,
        _pre_launch: L,
    ) -> anyhow::Result<()> {
        anyhow::bail!("rustcoalescence must be compiled to support at least one algorithm.")
    }
}
