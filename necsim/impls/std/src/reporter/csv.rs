use std::{
    fs::File,
    path::{Path, PathBuf},
};

use csv::{Writer, WriterBuilder};

use necsim_core::{
    event::Event,
    landscape::IndexedLocation,
    lineage::GlobalLineageReference,
    reporter::{EventFilter, Reporter},
};

#[allow(clippy::module_name_repetitions)]
pub struct CsvReporter {
    output: PathBuf,
    writer: Option<Writer<File>>,
}

impl EventFilter for CsvReporter {
    const REPORT_DISPERSAL: bool = true;
    const REPORT_SPECIATION: bool = true;
}

impl Reporter for CsvReporter {
    fn report_event(&mut self, event: &Event) {
        let output = &self.output;
        let writer = self.writer.get_or_insert_with(|| {
            let mut writer = WriterBuilder::new()
                .from_path(output)
                .unwrap_or_else(|_| panic!("Could not open {:?}", output));

            let _ = writer.write_record(&["reference", "time", "x", "y", "index"]);

            writer
        });

        let _ = Self::write_event(
            writer,
            event.global_lineage_reference(),
            event.time(),
            event.origin(),
        );
    }
}

impl CsvReporter {
    #[must_use]
    pub fn new(path: &Path) -> Self {
        Self {
            output: path.to_owned(),
            writer: None,
        }
    }

    pub fn finish(self) {
        std::mem::drop(self)
    }

    fn write_event(
        writer: &mut Writer<File>,
        reference: &GlobalLineageReference,
        event_time: f64,
        indexed_location: &IndexedLocation,
    ) -> csv::Result<()> {
        writer.write_record(&[
            &reference.to_string(),
            &event_time.to_string(),
            &indexed_location.location().x().to_string(),
            &indexed_location.location().y().to_string(),
            &indexed_location.index().to_string(),
        ])
    }
}
