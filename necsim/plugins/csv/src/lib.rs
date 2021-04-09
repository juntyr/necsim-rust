use std::{
    fmt,
    fs::{File, OpenOptions},
    io::{BufWriter, Write},
    path::{Path, PathBuf},
};

use serde::Deserialize;

use necsim_core::{
    impl_report, landscape::IndexedLocation, lineage::GlobalLineageReference, reporter::Reporter,
};

necsim_plugins_core::export_plugin!(Csv => CsvReporter);

#[allow(clippy::module_name_repetitions)]
#[derive(Deserialize)]
pub struct CsvReporter {
    output: PathBuf,
    #[serde(skip)]
    writer: Option<BufWriter<File>>,
}

impl fmt::Debug for CsvReporter {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt.debug_struct("CsvReporter")
            .field("output", &self.output)
            .finish()
    }
}

impl Reporter for CsvReporter {
    impl_report!(speciation(&mut self, event: Unused) -> Used {
        event.use_in(|event| {
            self.write_event(&event.global_lineage_reference, event.time, &event.origin, 's')
        })
    });

    impl_report!(dispersal(&mut self, event: Unused) -> Used {
        event.use_in(|event| {
            self.write_event(&event.global_lineage_reference, event.time, &event.origin, 'd')
        })
    });

    impl_report!(progress(&mut self, remaining: Unused) -> Unused {
        remaining.ignore()
    });
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
        &mut self,
        reference: &GlobalLineageReference,
        time: f64,
        origin: &IndexedLocation,
        r#type: char,
    ) {
        let output = &self.output;

        let writer = self.writer.get_or_insert_with(|| {
            let file = OpenOptions::new()
                .create(true)
                .truncate(true)
                .write(true)
                .open(output)
                .unwrap_or_else(|_| panic!("Could not open {:?}", output));

            let mut writer = BufWriter::new(file);

            std::mem::drop(writeln!(writer, "reference,time,x,y,index,type"));

            writer
        });

        std::mem::drop(writeln!(
            writer,
            "{},{},{},{},{},{}",
            reference,
            time,
            origin.location().x(),
            origin.location().y(),
            origin.index(),
            r#type,
        ));
    }
}
