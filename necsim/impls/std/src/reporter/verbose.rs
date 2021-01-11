use necsim_core::{
    event::Event,
    reporter::{EventFilter, Reporter},
};

#[allow(clippy::module_name_repetitions)]
pub struct VerboseReporter(());

impl EventFilter for VerboseReporter {
    const REPORT_DISPERSAL: bool = true;
    const REPORT_SPECIATION: bool = true;
}

impl Reporter for VerboseReporter {
    fn report_event(&mut self, event: &Event) {
        println!("{:#?}", event)
    }
}

impl Default for VerboseReporter {
    fn default() -> Self {
        Self(())
    }
}
