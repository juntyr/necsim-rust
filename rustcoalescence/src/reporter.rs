use necsim_impls_no_std::reporter::{GuardedReporter, ReporterContext};

use necsim_impls_std::reporter::biodiversity::BiodiversityReporter;
// use necsim_impls_std::reporter::events::EventReporter;
use necsim_impls_std::reporter::{
    execution_time::ExecutionTimeReporter, progress::ProgressReporter,
};

pub struct RustcoalescenceReporterContext(());

impl Default for RustcoalescenceReporterContext {
    fn default() -> Self {
        Self(())
    }
}

impl RustcoalescenceReporterContext {
    pub fn finalise(reporter_group: <Self as ReporterContext>::Reporter) {
        let biodiversity_reporter;
        let execution_time_reporter;
        let progress_reporter;

        // IV. Ungroup the reporters
        ReporterUnGroup! {reporter_group => [
            biodiversity_reporter,
            // event_reporter,
            execution_time_reporter,
            progress_reporter
        ]};

        // V. Output the simulation result and report summaries

        let execution_time = execution_time_reporter.execution_time();
        progress_reporter.finish();

        // event_reporter.report();

        if let Some(execution_time) = execution_time {
            info!(
                "The simulation took:\n - initialisation: {}s\n - execution: {}s\n - cleanup: {}s",
                execution_time.initialisation.as_secs_f32(),
                execution_time.execution.as_secs_f32(),
                execution_time.cleanup.as_secs_f32()
            );
        } else {
            info!("The simulation was not executed.");
        }

        info!(
            "The simulation resulted in a biodiversity of {} unique species.",
            biodiversity_reporter.biodiversity()
        );
    }
}

impl ReporterContext for RustcoalescenceReporterContext {
    type Finaliser = fn(Self::Reporter);
    type Reporter = ReporterGroupType![
        BiodiversityReporter,
        ExecutionTimeReporter,
        ProgressReporter
    ];

    fn build_guarded(self) -> GuardedReporter<Self::Reporter, Self::Finaliser> {
        // I. Initialise the reporters

        let biodiversity_reporter = BiodiversityReporter::default();
        // let mut event_reporter = EventReporter::default();
        let execution_time_reporter = ExecutionTimeReporter::default();
        let progress_reporter = ProgressReporter::default();

        // II. Group the reporters into one static group type

        let reporter_group = ReporterGroup![
            biodiversity_reporter,
            // event_reporter,
            execution_time_reporter,
            progress_reporter
        ];

        // III. Return the guarded reporter group
        GuardedReporter::from(reporter_group, Self::finalise)
    }
}
