use necsim_impls_no_std::reporter::ReporterContext;

use necsim_impls_std::reporter::biodiversity::BiodiversityReporter;
// use necsim_impls_std::reporter::events::EventReporter;
use necsim_impls_std::reporter::{
    execution_time::ExecutionTimeReporter, progress::ProgressReporter,
};

pub struct RustcoalescenceReporterContext {
    estimated_total_lineages: u64,
}

impl RustcoalescenceReporterContext {
    pub fn new(estimated_total_lineages: u64) -> Self {
        Self {
            estimated_total_lineages,
        }
    }
}

impl ReporterContext for RustcoalescenceReporterContext {
    type Reporter = ReporterGroupType![
        BiodiversityReporter,
        ExecutionTimeReporter,
        ProgressReporter
    ];

    fn with_reporter<O, F: FnOnce(&mut Self::Reporter) -> O>(self, inner: F) -> O {
        // I. Initialise the reporters

        let mut biodiversity_reporter = BiodiversityReporter::default();
        // let mut event_reporter = EventReporter::default();
        let mut execution_time_reporter = ExecutionTimeReporter::default();
        let mut progress_reporter = ProgressReporter::new(self.estimated_total_lineages);

        // II. Group the reporters into one static group type

        let mut reporter_group = ReporterGroup![
            biodiversity_reporter,
            // event_reporter,
            execution_time_reporter,
            progress_reporter
        ];

        // III. Lend the reporter to the inner simulation closure

        let result = inner(&mut reporter_group);

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

        // VI. Return the result of the inner simulation closure

        result
    }
}
