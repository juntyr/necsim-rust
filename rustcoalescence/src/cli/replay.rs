use anyhow::Result;

use necsim_core::reporter::Reporter;

use necsim_impls_no_std::reporter::ReporterContext;
use necsim_impls_std::event_log::replay::EventLogReplay;

use crate::args::ReplayCommandArgs;

#[allow(clippy::module_name_repetitions, clippy::needless_pass_by_value)]
pub fn replay_with_logger<R: ReporterContext>(
    replay_args: ReplayCommandArgs,
    reporter_context: R,
) -> Result<()> {
    anyhow::ensure!(
        !replay_args.events.is_empty(),
        "The replay command must be given at least one event log."
    );

    info!("Starting event replay ...");

    let mut reporter = reporter_context.build_guarded();

    for event in EventLogReplay::try_new(&replay_args.events, 100_000)? {
        reporter.report_progress(1_u64);
        reporter.report_event(&event);
        reporter.report_progress(0_u64);
    }

    std::mem::drop(reporter);

    info!("The event replay has completed.");

    Ok(())
}
