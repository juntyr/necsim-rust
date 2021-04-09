use anyhow::Result;
use log::LevelFilter;

use necsim_impls_no_std::partitioning::monolithic::live::{
    LiveMonolithicLocalPartition, LiveMonolithicPartitioning,
};
use necsim_impls_std::partitioning::monolithic::recorded::RecordedMonolithicLocalPartition;
use necsim_plugins_core::match_any_reporter_plugin_vec;

use crate::{
    args::{SimulateArgs, SimulateCommandArgs},
    reporter::DynamicReporterContext,
};

#[allow(clippy::module_name_repetitions)]
pub fn simulate_with_logger_monolithic(simulate_args: SimulateCommandArgs) -> Result<()> {
    log::set_max_level(LevelFilter::Info);

    let simulate_args =
        SimulateArgs::try_parse(simulate_args, &LiveMonolithicPartitioning::default())?;
    info!("Parsed simulation arguments:\n{:#?}", simulate_args);

    match_any_reporter_plugin_vec!(simulate_args.reporters => |reporter| {
        // Initialise the local partition and the simulation
        match simulate_args.event_log {
            Some(event_log) => super::simulate_with_logger(
                Box::new(
                    RecordedMonolithicLocalPartition::from_context_and_recorder(
                        DynamicReporterContext::new(reporter),
                        event_log,
                    ),
                ),
                simulate_args.common,
                simulate_args.scenario,
            ),
            None => super::simulate_with_logger(
                Box::new(LiveMonolithicLocalPartition::from_context(DynamicReporterContext::new(reporter))),
                simulate_args.common,
                simulate_args.scenario,
            ),
        }
    })
}
