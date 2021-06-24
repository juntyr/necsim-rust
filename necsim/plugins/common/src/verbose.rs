use std::fmt;

use necsim_core::{impl_report, reporter::Reporter};

#[allow(clippy::module_name_repetitions)]
pub struct VerboseReporter(());

impl fmt::Debug for VerboseReporter {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_struct("VerboseReporter").finish()
    }
}

impl<'de> serde::Deserialize<'de> for VerboseReporter {
    fn deserialize<D: serde::Deserializer<'de>>(_deserializer: D) -> Result<Self, D::Error> {
        Ok(Self::default())
    }
}

impl Reporter for VerboseReporter {
    impl_report!(speciation(&mut self, speciation: Used) {
        info!("{:#?}", speciation);
    });

    impl_report!(dispersal(&mut self, dispersal: Used) {
        info!("{:#?}", dispersal);
    });

    impl_report!(progress(&mut self, remaining: Used) {
        info!("Remaining: {}", remaining);
    });
}

impl Default for VerboseReporter {
    fn default() -> Self {
        Self(())
    }
}
