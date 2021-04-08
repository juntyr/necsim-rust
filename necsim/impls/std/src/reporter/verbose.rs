use std::fmt;

use necsim_core::{impl_report, reporter::Reporter};

#[allow(clippy::module_name_repetitions)]
pub struct VerboseReporter(());

impl fmt::Debug for VerboseReporter {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt.debug_struct("VerboseReporter").finish()
    }
}

impl<'de> serde::Deserialize<'de> for VerboseReporter {
    fn deserialize<D: serde::Deserializer<'de>>(_deserializer: D) -> Result<Self, D::Error> {
        Ok(Self::default())
    }
}

impl Reporter for VerboseReporter {
    impl_report!(speciation(&mut self, event: Unused) -> Used {
        event.use_in(|event| {
            info!("{:#?}", event)
        })
    });

    impl_report!(dispersal(&mut self, event: Unused) -> Used {
        event.use_in(|event| {
            info!("{:#?}", event)
        })
    });

    impl_report!(progress(&mut self, remaining: Unused) -> Used {
        remaining.use_in(|remaining| {
            info!("Remaining({})", remaining)
        })
    });
}

impl Default for VerboseReporter {
    fn default() -> Self {
        Self(())
    }
}
