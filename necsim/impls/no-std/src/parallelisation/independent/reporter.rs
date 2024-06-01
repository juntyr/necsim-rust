use core::{fmt, marker::PhantomData};

use necsim_core::{impl_report, reporter::Reporter};

use necsim_partitioning_core::LocalPartition;

pub struct IgnoreProgressReporterProxy<'p, R: Reporter, P: LocalPartition<R>> {
    local_partition: &'p mut P,
    _marker: PhantomData<R>,
}

impl<'p, R: Reporter, P: LocalPartition<R>> fmt::Debug for IgnoreProgressReporterProxy<'p, R, P> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_struct(stringify!(IgnoreProgressReporterProxy))
            .finish()
    }
}

impl<'p, R: Reporter, P: LocalPartition<R>> Reporter for IgnoreProgressReporterProxy<'p, R, P> {
    impl_report!(speciation(&mut self, speciation: MaybeUsed<
        <<P as LocalPartition<R>>::Reporter as Reporter
    >::ReportSpeciation>) {
        self.local_partition.get_reporter().report_speciation(speciation.into());
    });

    impl_report!(dispersal(&mut self, dispersal: MaybeUsed<
        <<P as LocalPartition<R>>::Reporter as Reporter
    >::ReportDispersal>) {
        self.local_partition.get_reporter().report_dispersal(dispersal.into());
    });

    impl_report!(progress(&mut self, _progress: Ignored) {});
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
            .report_progress(&remaining.into());
    }

    pub fn local_partition(&mut self) -> &mut P {
        self.local_partition
    }
}
