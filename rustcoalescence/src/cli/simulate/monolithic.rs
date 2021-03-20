use std::convert::TryInto;

use anyhow::Result;
use log::LevelFilter;

use necsim_impls_no_std::{
    partitioning::monolithic::live::LiveMonolithicLocalPartition, reporter::ReporterContext,
};
use necsim_impls_std::{
    event_log::recorder::EventLogRecorder,
    partitioning::monolithic::recorded::RecordedMonolithicLocalPartition,
};

use crate::{
    args::{SimulateArgs, SimulateCommandArgs},
    reporter::RustcoalescenceReporterContext,
};

#[allow(clippy::module_name_repetitions)]
pub fn simulate_with_logger_monolithic(simulate_args: SimulateCommandArgs) -> Result<()> {
    log::set_max_level(LevelFilter::Info);

    let guard_reporter = RustcoalescenceReporterContext::new(true).build_guarded();

    let simulate_args: SimulateArgs = simulate_args.try_into()?;
    info!("Parsed simulation arguments:\n{:#?}", simulate_args);

    // Initialise the local partition and the simulation
    match simulate_args.event_log {
        Some(event_log_path) => super::simulate_with_logger::<RustcoalescenceReporterContext, _>(
            Box::new(
                RecordedMonolithicLocalPartition::from_reporter_and_recorder(
                    guard_reporter,
                    EventLogRecorder::try_new(&event_log_path)?,
                ),
            ),
            simulate_args.common,
            simulate_args.scenario,
        ),
        None => super::simulate_with_logger::<RustcoalescenceReporterContext, _>(
            Box::new(LiveMonolithicLocalPartition::from_reporter(guard_reporter)),
            simulate_args.common,
            simulate_args.scenario,
        ),
    }
}
