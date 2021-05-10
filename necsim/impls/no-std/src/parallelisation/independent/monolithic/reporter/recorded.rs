use core::{fmt, marker::PhantomData};
use necsim_core_bond::NonNegativeF64;

use necsim_core::{
    impl_report,
    reporter::{used::Unused, Reporter},
};

use necsim_partitioning_core::LocalPartition;

use super::WaterLevelReporterProxy;

#[allow(clippy::module_name_repetitions)]
pub struct RecordedWaterLevelReporterProxy<'p, R: Reporter, P: LocalPartition<R>> {
    water_level: NonNegativeF64,

    local_partition: &'p mut P,
    _marker: PhantomData<R>,
}

impl<'p, R: Reporter, P: LocalPartition<R>> fmt::Debug
    for RecordedWaterLevelReporterProxy<'p, R, P>
{
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        struct EventBufferLen(usize);

        impl fmt::Debug for EventBufferLen {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                write!(f, "Vec<PackedEvent; {}>", self.0)
            }
        }

        fmt.debug_struct("RecordedWaterLevelReporterProxy")
            .field("water_level", &self.water_level)
            .finish()
    }
}

impl<'p, R: Reporter, P: LocalPartition<R>> Reporter for RecordedWaterLevelReporterProxy<'p, R, P> {
    impl_report!(speciation(&mut self, event: Unused) -> MaybeUsed<R::ReportSpeciation> {
        event.maybe_use_in(|event| {
            self.local_partition.get_reporter().report_speciation(Unused::new(&event));
        })
    });

    impl_report!(dispersal(&mut self, event: Unused) -> MaybeUsed<R::ReportDispersal> {
        event.maybe_use_in(|event| {
            self.local_partition.get_reporter().report_dispersal(Unused::new(&event));
        })
    });

    impl_report!(progress(&mut self, remaining: Unused) -> Unused {
        remaining.ignore()
    });
}

#[contract_trait]
impl<'p, R: Reporter, P: LocalPartition<R>> WaterLevelReporterProxy<'p, R, P>
    for RecordedWaterLevelReporterProxy<'p, R, P>
{
    fn new(_capacity: usize, local_partition: &'p mut P) -> Self {
        info!("Events will be reported using the recorded water-level algorithm ...");

        Self {
            water_level: NonNegativeF64::zero(),

            local_partition,
            _marker: PhantomData::<R>,
        }
    }

    fn water_level(&self) -> NonNegativeF64 {
        self.water_level
    }

    fn advance_water_level(&mut self, water_level: NonNegativeF64) {
        self.water_level = water_level;
    }

    fn local_partition(&mut self) -> &mut P {
        self.local_partition
    }
}
