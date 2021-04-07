use core::u64;

use crate::{impl_report, reporter::Reporter};

#[allow(clippy::module_name_repetitions)]
pub struct NullReporter;

impl Reporter for NullReporter {
    impl_report!(speciation(&mut self, event: Unused) -> Unused {
        event.ignore()
    });

    impl_report!(dispersal(&mut self, event: Unused) -> Unused {
        event.ignore()
    });

    impl_report!(progress(&mut self, remaining: Unused) -> Unused {
        remaining.ignore()
    });
}
