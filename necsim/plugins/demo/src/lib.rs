#![deny(clippy::pedantic)]

use std::{
    fmt,
    io::{self, Read, Write},
};

use necsim_core::{impl_report, reporter::Reporter};

necsim_plugins_core::export_plugin!(Demo => DemoReporter);

#[allow(clippy::module_name_repetitions)]
pub struct DemoReporter {
    initialised: bool,
}

impl fmt::Debug for DemoReporter {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_struct("DemoReporter").finish()
    }
}

impl<'de> serde::Deserialize<'de> for DemoReporter {
    fn deserialize<D: serde::Deserializer<'de>>(_deserializer: D) -> Result<Self, D::Error> {
        Ok(Self::default())
    }
}

impl Reporter for DemoReporter {
    impl_report!(speciation(&mut self, speciation: Used) {
        self.check_initialised();

        std::mem::drop(Self::confirm_continue(&format!(
            "{:>5.2}: <{}> speciates              at ({},{}):{} ...",
            speciation.event_time.get(),
            speciation.global_lineage_reference,
            speciation.origin.location().x(),
            speciation.origin.location().y(),
            speciation.origin.index(),
        )));
    });

    impl_report!(dispersal(&mut self, dispersal: Used) {
        self.check_initialised();

        std::mem::drop(Self::confirm_continue(&format!(
            "{:>5.2}: <{}> disperses from ({},{}):{} to ({},{}):{} ...",
            dispersal.event_time.get(),
            dispersal.global_lineage_reference,
            dispersal.origin.location().x(),
            dispersal.origin.location().y(),
            dispersal.origin.index(),
            dispersal.target.location().x(),
            dispersal.target.location().y(),
            dispersal.target.index(),
        )));
    });

    impl_report!(progress(&mut self, _remaining: Ignored) {});
}

impl Default for DemoReporter {
    fn default() -> Self {
        Self { initialised: false }
    }
}

impl DemoReporter {
    fn check_initialised(&mut self) {
        if !self.initialised && atty::is(atty::Stream::Stdin) {
            println!("{:=^80}", "");
            println!("={: ^78}=", "Starting Interactive Event Prompt ...");
            println!("={: ^78}=", "(Press ENTER to continue)");
            println!("{:=^80}", "");

            std::mem::drop(Self::confirm_continue(""));
        }

        self.initialised = true;
    }

    fn confirm_continue(message: &str) -> io::Result<()> {
        let mut stdout = io::stdout();

        if atty::is(atty::Stream::Stdin) {
            write!(stdout, "{}", message)?;
            stdout.flush()?;

            io::stdin().read(&mut [0_u8; 1]).map(|_| ())
        } else {
            writeln!(stdout, "{}", message)?;
            stdout.flush()
        }
    }
}
