use std::time::Instant;

use necsim_core::{impl_finalise, impl_report, reporter::Reporter};

#[allow(clippy::module_name_repetitions)]
#[derive(Debug)]
pub struct ExecutionTimeReporter {
    init_time: Instant,
    start_time: Option<Instant>,
    end_time: Option<Instant>,
}

impl<'de> serde::Deserialize<'de> for ExecutionTimeReporter {
    fn deserialize<D: serde::Deserializer<'de>>(_deserializer: D) -> Result<Self, D::Error> {
        Ok(Self::default())
    }
}

impl Reporter for ExecutionTimeReporter {
    impl_report!(speciation(&mut self, event: Unused) -> Unused {
        event.ignore()
    });

    impl_report!(dispersal(&mut self, event: Unused) -> Unused {
        event.ignore()
    });

    impl_report!(
        #[debug_ensures(self.start_time.is_some(), "start_time is set after first call")]
        progress(&mut self, remaining: Unused) -> Used {
            remaining.use_in(|remaining| {
                if self.start_time.is_none() {
                    self.start_time = Some(Instant::now());
                }

                if *remaining == 0 {
                    self.end_time = Some(Instant::now());
                }
            })
        }
    );

    impl_finalise!((self) {
        if let (Some(start_time), Some(end_time)) = (self.start_time, self.end_time) {
            info!(
                "The simulation took:\n - initialisation: {}s\n - execution: {}s\n - cleanup: {}s",
                (start_time - self.init_time).as_secs_f32(),
                (end_time - start_time).as_secs_f32(),
                end_time.elapsed().as_secs_f32()
            )
        }
    });

    fn initialise(&mut self) -> Result<(), String> {
        self.init_time = Instant::now();
        self.start_time = None;
        self.end_time = None;

        Ok(())
    }
}

impl Default for ExecutionTimeReporter {
    #[debug_ensures(ret.start_time.is_none(), "initialises start_time to None")]
    #[debug_ensures(ret.end_time.is_none(), "initialises end_time to None")]
    fn default() -> Self {
        Self {
            init_time: Instant::now(),
            start_time: None,
            end_time: None,
        }
    }
}
