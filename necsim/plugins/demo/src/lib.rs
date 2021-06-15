#![deny(clippy::pedantic)]

use std::{collections::HashSet, fmt};

use serde::Deserialize;

use necsim_core::{impl_report, landscape::Location, reporter::Reporter};

necsim_plugins_core::export_plugin!(Demo => DemoReporter);

#[allow(clippy::module_name_repetitions)]
#[derive(Deserialize)]
#[serde(from = "DemoReporterArgs")]
pub struct DemoReporter {
    ignore: HashSet<Location>,
    initialised: bool,
}

impl fmt::Debug for DemoReporter {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_struct("DemoReporter")
            .field("ignore", &self.ignore)
            .finish()
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(default)]
struct DemoReporterArgs {
    ignore: HashSet<Location>,
}

impl Default for DemoReporterArgs {
    fn default() -> Self {
        Self {
            ignore: HashSet::new(),
        }
    }
}

impl From<DemoReporterArgs> for DemoReporter {
    fn from(args: DemoReporterArgs) -> Self {
        Self {
            ignore: args.ignore,
            initialised: false,
        }
    }
}

impl Reporter for DemoReporter {
    impl_report!(speciation(&mut self, speciation: Used) {
        self.check_initialised();

        if self.ignore.contains(speciation.origin.location()) {
            return
        }

        println!(
            "{:>5.2}: <{}> speciates              at ({},{}):{} ...",
            speciation.event_time.get(),
            speciation.global_lineage_reference,
            speciation.origin.location().x(),
            speciation.origin.location().y(),
            speciation.origin.index(),
        );
    });

    impl_report!(dispersal(&mut self, dispersal: Used) {
        self.check_initialised();

        if self.ignore.contains(dispersal.target.location()) {
            return
        }

        println!(
            "{:>5.2}: <{}> disperses from ({},{}):{} to ({},{}):{} ...",
            dispersal.event_time.get(),
            dispersal.global_lineage_reference,
            dispersal.origin.location().x(),
            dispersal.origin.location().y(),
            dispersal.origin.index(),
            dispersal.target.location().x(),
            dispersal.target.location().y(),
            dispersal.target.index(),
        );
    });

    impl_report!(progress(&mut self, _remaining: Ignored) {});
}

impl DemoReporter {
    fn check_initialised(&mut self) {
        if !self.initialised {
            println!("{:=^80}", "");
            println!("={: ^78}=", "Starting Event Report ...");
            println!("{:=^80}", "");
        }

        self.initialised = true;
    }
}
