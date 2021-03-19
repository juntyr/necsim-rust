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
    args::{RustcoalescenceArgs, SimulateArgs},
    reporter::RustcoalescenceReporterContext,
};

#[allow(clippy::module_name_repetitions)]
pub fn simulate_with_logger_monolithic(
    args: &RustcoalescenceArgs,
    simulate_args: &SimulateArgs,
) -> Result<()> {
    #[allow(clippy::manual_map)]
    let recorder = match simulate_args.common_args().event_log().as_deref() {
        Some(event_log_path) => Some(EventLogRecorder::try_new(event_log_path)?),
        None => None,
    };

    log::set_max_level(LevelFilter::Info);

    info!("Parsed arguments:\n{:#?}", args);

    let guard_reporter = RustcoalescenceReporterContext::new(true).build_guarded();

    // Initialise the local partition and the simulation
    match recorder {
        Some(recorder) => super::simulate_with_logger::<RustcoalescenceReporterContext, _>(
            Box::new(
                RecordedMonolithicLocalPartition::from_reporter_and_recorder(
                    guard_reporter,
                    recorder,
                ),
            ),
            simulate_args,
        ),
        None => super::simulate_with_logger::<RustcoalescenceReporterContext, _>(
            Box::new(LiveMonolithicLocalPartition::from_reporter(guard_reporter)),
            simulate_args,
        ),
    }
}
