use necsim_impls_no_std::reporter::{GuardedReporter, ReporterContext};

use necsim_impls_std::reporter::{
    biodiversity::BiodiversityReporter, execution_time::ExecutionTimeReporter,
    progress::ProgressReporter,
};

// use necsim_impls_std::reporter::csv::CsvReporter;

#[derive(Debug)]
pub struct RustcoalescenceReporterContext {
    report_analysis: bool,
}

impl RustcoalescenceReporterContext {
    pub fn new(report_analysis: bool) -> Self {
        Self { report_analysis }
    }
}

impl RustcoalescenceReporterContext {
    pub fn finalise<const REPORT_ANALYSIS: bool>(
        reporter_group: <Self as ReporterContext>::Reporter,
    ) {
        let biodiversity_reporter;
        // let csv_reporter;
        let execution_time_reporter;
        let progress_reporter;

        // IV. Ungroup the reporters
        ReporterUnGroup! {reporter_group => [
            biodiversity_reporter,
            //csv_reporter,
            execution_time_reporter,
            progress_reporter
        ]};

        // V. Output the simulation result and report summaries

        let execution_time = execution_time_reporter.execution_time();
        progress_reporter.finish();

        // csv_reporter.finish();

        if let Some(execution_time) = execution_time {
            info!(
                "The simulation took:\n - initialisation: {}s\n - execution: {}s\n - cleanup: {}s",
                execution_time.initialisation.as_secs_f32(),
                execution_time.execution.as_secs_f32(),
                execution_time.cleanup.as_secs_f32()
            );
        }

        if REPORT_ANALYSIS {
            let biodiversity = biodiversity_reporter.biodiversity();

            if biodiversity > 0 {
                info!(
                    "The simulation resulted in a biodiversity of {} unique species.",
                    biodiversity
                );
            }
        }
    }
}

impl ReporterContext for RustcoalescenceReporterContext {
    type Finaliser = fn(Self::Reporter);
    type Reporter = ReporterGroupType![
        BiodiversityReporter,
        // CsvReporter,
        ExecutionTimeReporter,
        ProgressReporter
    ];

    fn build_guarded(self) -> GuardedReporter<Self::Reporter, Self::Finaliser> {
        // I. Initialise the reporters

        let biodiversity_reporter = BiodiversityReporter::default();
        // let csv_reporter = CsvReporter::new(&std::path::PathBuf::from("events.csv"));
        let execution_time_reporter = ExecutionTimeReporter::default();
        let progress_reporter = ProgressReporter::default();

        // II. Group the reporters into one static group type
        let reporter_group = ReporterGroup![
            biodiversity_reporter,
            // csv_reporter,
            execution_time_reporter,
            progress_reporter
        ];

        // III. Return the guarded reporter group
        if self.report_analysis {
            GuardedReporter::from(reporter_group, Self::finalise::<true>)
        } else {
            GuardedReporter::from(reporter_group, Self::finalise::<false>)
        }
    }
}
