use necsim_core::{impl_report, reporter::Reporter};

#[allow(clippy::module_name_repetitions)]
pub struct VerboseReporter(());

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
