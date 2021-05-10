use core::{fmt, marker::PhantomData};

use necsim_core::{
    impl_report,
    reporter::{used::Unused, Reporter},
};

use crate::partitioning::LocalPartition;

pub struct IgnoreProgressReporterProxy<'p, R: Reporter, P: LocalPartition<R>> {
    local_partition: &'p mut P,
    _marker: PhantomData<R>,
}

impl<'p, R: Reporter, P: LocalPartition<R>> fmt::Debug for IgnoreProgressReporterProxy<'p, R, P> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_struct("IgnoreProgressReporterProxy").finish()
    }
}

impl<'p, R: Reporter, P: LocalPartition<R>> Reporter for IgnoreProgressReporterProxy<'p, R, P> {
    impl_report!(speciation(&mut self, event: Unused) -> MaybeUsed<
        <<P as LocalPartition<R>>::Reporter as Reporter
    >::ReportSpeciation> {
        self.local_partition.get_reporter().report_speciation(event)
    });

    impl_report!(dispersal(&mut self, event: Unused) -> MaybeUsed<
        <<P as LocalPartition<R>>::Reporter as Reporter
    >::ReportDispersal> {
        self.local_partition.get_reporter().report_dispersal(event)
    });

    impl_report!(progress(&mut self, remaining: Unused) -> Unused {
        remaining.ignore()
    });
}

impl<'p, R: Reporter, P: LocalPartition<R>> IgnoreProgressReporterProxy<'p, R, P> {
    pub fn from(local_partition: &'p mut P) -> Self {
        Self {
            local_partition,
            _marker: PhantomData::<R>,
        }
    }

    #[inline]
    pub fn report_total_progress(&mut self, remaining: u64) {
        self.local_partition
            .get_reporter()
            .report_progress(Unused::new(&remaining));
    }

    pub fn local_partition(&mut self) -> &mut P {
        self.local_partition
    }
}
