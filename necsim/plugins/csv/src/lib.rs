#![deny(clippy::pedantic)]

use std::{
    convert::TryFrom,
    fmt,
    fs::{File, OpenOptions},
    io::{self, BufWriter, Write},
    path::PathBuf,
};

use serde::Deserialize;

use necsim_core::{
    impl_report, landscape::IndexedLocation, lineage::GlobalLineageReference, reporter::Reporter,
};

necsim_plugins_core::export_plugin!(Csv => CsvReporter);

#[allow(clippy::module_name_repetitions)]
#[derive(Deserialize)]
#[serde(try_from = "CsvReporterArgs")]
pub struct CsvReporter {
    output: PathBuf,
    writer: Option<BufWriter<File>>,
}

impl fmt::Debug for CsvReporter {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_struct("CsvReporter")
            .field("output", &self.output)
            .finish()
    }
}

#[derive(Deserialize)]
struct CsvReporterArgs {
    output: PathBuf,
}

impl TryFrom<CsvReporterArgs> for CsvReporter {
    type Error = io::Error;

    fn try_from(args: CsvReporterArgs) -> Result<Self, Self::Error> {
        // Preliminary argument parsing check if the output is a writable file
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&args.output)?;
        std::mem::drop(file);

        Ok(Self {
            output: args.output,
            writer: None,
        })
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

    fn finalise_impl(&mut self) {
        if let Some(writer) = &mut self.writer {
            std::mem::drop(writer.flush());
        }
    }
}

impl CsvReporter {
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
                .unwrap_or_else(|_| panic!("Could not write to the output path {:?}", output));

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
