use crate::event_generator::Event;

pub trait Reporter {
    fn report_event(&mut self, event: &Event);
}

#[allow(clippy::module_name_repetitions)]
pub struct NullReporter;

impl Reporter for NullReporter {
    fn report_event(&mut self, _event: &Event) {
        // no-op
    }
}

#[allow(clippy::module_name_repetitions)]
pub struct ReporterGroup<'r> {
    reporters: &'r mut [Box<dyn Reporter>],
}

impl<'r> Reporter for ReporterGroup<'r> {
    fn report_event(&mut self, event: &Event) {
        self.reporters
            .iter_mut()
            .for_each(|reporter| reporter.report_event(event))
    }
}

impl<'r> ReporterGroup<'r> {
    pub fn new(reporters: &'r mut [Box<dyn Reporter>]) -> Self {
        Self { reporters }
    }
}
