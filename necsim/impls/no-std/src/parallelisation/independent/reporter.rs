use core::{fmt, marker::PhantomData};

use necsim_core::{impl_report, reporter::Reporter};

use necsim_partitioning_core::LocalPartition;

pub struct IgnoreProgressReporterProxy<'l, 'p, R: Reporter, P: LocalPartition<'p, R>> {
    local_partition: &'l mut P,
    _marker: PhantomData<(&'p (), R)>,
}

impl<'l, 'p, R: Reporter, P: LocalPartition<'p, R>> fmt::Debug
    for IgnoreProgressReporterProxy<'l, 'p, R, P>
{
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_struct(stringify!(IgnoreProgressReporterProxy))
            .finish()
    }
}

impl<'l, 'p, R: Reporter, P: LocalPartition<'p, R>> Reporter
    for IgnoreProgressReporterProxy<'l, 'p, R, P>
{
    impl_report!(speciation(&mut self, speciation: MaybeUsed<
        <<P as LocalPartition<'p, R>>::Reporter as Reporter
    >::ReportSpeciation>) {
        self.local_partition.get_reporter().report_speciation(speciation.into());
    });

    impl_report!(dispersal(&mut self, dispersal: MaybeUsed<
        <<P as LocalPartition<'p, R>>::Reporter as Reporter
    >::ReportDispersal>) {
        self.local_partition.get_reporter().report_dispersal(dispersal.into());
    });

    impl_report!(progress(&mut self, _progress: Ignored) {});
}

impl<'l, 'p, R: Reporter, P: LocalPartition<'p, R>> IgnoreProgressReporterProxy<'l, 'p, R, P> {
    pub fn from(local_partition: &'l mut P) -> Self {
        Self {
            local_partition,
            _marker: PhantomData::<(&'p (), R)>,
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
