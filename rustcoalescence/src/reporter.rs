use array2d::Array2D;

use necsim_core::cogs::{Habitat, LineageReference};
use necsim_impls_no_std::reporter::ReporterContext;

use necsim_impls_std::reporter::biodiversity::BiodiversityReporter;
// use necsim_impls_std::reporter::events::EventReporter;
// use necsim_impls_std::reporter::execution_time::ExecutionTimeReporter;
use necsim_impls_std::reporter::progress::ProgressReporter;

use crate::args::CommandLineArguments;

pub struct RustcoalescenceReporterContext {
    estimated_total_lineages: u64,
}

impl RustcoalescenceReporterContext {
    pub fn new(args: &CommandLineArguments, habitat: &Array2D<u32>) -> Self {
        let total_habitat = habitat
            .elements_row_major_iter()
            .map(|x| u64::from(*x))
            .sum::<u64>();

        #[allow(clippy::cast_possible_truncation)]
        #[allow(clippy::cast_sign_loss)]
        #[allow(clippy::cast_precision_loss)]
        let estimated_total_lineages =
            ((total_habitat as f64) * args.sample_percentage()).ceil() as u64;

        Self {
            estimated_total_lineages,
        }
    }
}

impl ReporterContext for RustcoalescenceReporterContext {
    type Reporter<H: Habitat, R: LineageReference<H>> = ReporterGroupType! {<H, R>[
        BiodiversityReporter,
        ProgressReporter
    ]};

    fn with_reporter<
        O,
        H: Habitat,
        R: LineageReference<H>,
        F: FnOnce(&mut Self::Reporter<H, R>) -> O,
    >(
        self,
        inner: F,
    ) -> O {
        // I. Initialise the reporters

        let mut biodiversity_reporter = BiodiversityReporter::default();
        // let mut event_reporter = EventReporter::default();
        // let mut execution_time_reporter = ExecutionTimeReporter::default();
        let mut progress_reporter = ProgressReporter::new(self.estimated_total_lineages);

        // II. Group the reporters into one static group type

        let mut reporter_group = ReporterGroup![
            biodiversity_reporter,
            // event_reporter,
            // execution_time_reporter,
            progress_reporter
        ];

        // III. Lend the reporter to the inner simulation closure

        let result = inner(&mut reporter_group);

        // IV. Ungroup the reporters

        ReporterUnGroup! {reporter_group => [
            biodiversity_reporter,
            // event_reporter,
            // execution_time_reporter,
            progress_reporter
        ]};

        // V. Output the simulation result and report summaries

        // let execution_time = execution_time_reporter.execution_time();
        progress_reporter.finish();
        // event_reporter.report();
        // println!(
        // "The simulation took {}s to execute.",
        // execution_time.as_secs_f32()
        // );
        println!(
            "Simulation resulted with biodiversity of {} unique species.",
            biodiversity_reporter.biodiversity()
        );

        // VI. Return the result of the inner simulation closure

        result
    }
}
