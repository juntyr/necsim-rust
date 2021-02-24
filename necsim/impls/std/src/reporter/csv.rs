use std::{
    fs::{File, OpenOptions},
    io::{BufWriter, Write},
    path::{Path, PathBuf},
};

use necsim_core::{
    event::{Event, EventType},
    reporter::{EventFilter, Reporter},
};

#[allow(clippy::module_name_repetitions)]
pub struct CsvReporter {
    output: PathBuf,
    writer: Option<BufWriter<File>>,
}

impl EventFilter for CsvReporter {
    const REPORT_DISPERSAL: bool = true;
    const REPORT_SPECIATION: bool = true;
}

impl Reporter for CsvReporter {
    fn report_event(&mut self, event: &Event) {
        let output = &self.output;

        let writer = self.writer.get_or_insert_with(|| {
            let file = OpenOptions::new()
                .create(true)
                .truncate(true)
                .write(true)
                .open(output)
                .unwrap_or_else(|_| panic!("Could not open {:?}", output));

            let mut writer = BufWriter::new(file);

            let _ = writeln!(writer, "reference,time,x,y,index,type");

            writer
        });

        let _ = writeln!(
            writer,
            "{},{},{},{},{},{}",
            event.global_lineage_reference(),
            event.time(),
            event.origin().location().x(),
            event.origin().location().y(),
            event.origin().index(),
            match event.r#type() {
                EventType::Speciation => 's',
                EventType::Dispersal { .. } => 'd',
            }
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
}
