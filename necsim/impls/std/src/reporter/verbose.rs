use necsim_core::{
    cogs::{Habitat, LineageReference},
    event::Event,
    reporter::{EventFilter, Reporter},
};

#[allow(clippy::module_name_repetitions)]
pub struct VerboseReporter(());

impl EventFilter for VerboseReporter {
    const REPORT_DISPERSAL: bool = true;
    const REPORT_SPECIATION: bool = true;
}

impl<H: Habitat, R: LineageReference<H>> Reporter<H, R> for VerboseReporter {
    fn report_event(&mut self, event: &Event<H, R>) {
        println!("{:#?}", event)
    }
}

impl Default for VerboseReporter {
    fn default() -> Self {
        Self(())
    }
}
