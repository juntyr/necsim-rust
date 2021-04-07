use std::marker::PhantomData;

use necsim_core::{
    event::{PackedEvent, TypedEvent},
    impl_report,
    reporter::{used::Unused, Reporter},
};

use necsim_impls_no_std::{partitioning::LocalPartition, reporter::ReporterContext};

pub struct PartitionReporterProxy<'p, R: ReporterContext, P: LocalPartition<R>> {
    local_partition: &'p mut P,
    event_buffer: Vec<PackedEvent>,
    _marker: PhantomData<R>,
}

impl<'p, R: ReporterContext, P: LocalPartition<R>> Reporter for PartitionReporterProxy<'p, R, P> {
    impl_report!(speciation(&mut self, event: Unused) -> MaybeUsed<
        <<P as LocalPartition<R>>::Reporter as Reporter
    >::ReportSpeciation> {
        event.maybe_use_in(|event| {
            self.event_buffer.push(event.clone().into())
        })
    });

    impl_report!(dispersal(&mut self, event: Unused) -> MaybeUsed<
        <<P as LocalPartition<R>>::Reporter as Reporter
    >::ReportDispersal> {
        event.maybe_use_in(|event| {
            self.event_buffer.push(event.clone().into())
        })
    });

    impl_report!(progress(&mut self, remaining: Unused) -> MaybeUsed<
        <<P as LocalPartition<R>>::Reporter as Reporter
    >::ReportProgress> {
        self.local_partition.get_reporter().report_progress(remaining)
    });
}

impl<'p, R: ReporterContext, P: LocalPartition<R>> PartitionReporterProxy<'p, R, P> {
    pub fn from(local_partition: &'p mut P) -> Self {
        Self {
            local_partition,
            event_buffer: Vec::new(),
            _marker: PhantomData::<R>,
        }
    }

    pub fn clear_events(&mut self) {
        self.event_buffer.clear()
    }

    pub fn report_events(&mut self) {
        for event in self.event_buffer.drain(..) {
            match event.into() {
                TypedEvent::Speciation(event) => {
                    self.local_partition
                        .get_reporter()
                        .report_speciation(Unused::new(&event));
                },
                TypedEvent::Dispersal(event) => {
                    self.local_partition
                        .get_reporter()
                        .report_dispersal(Unused::new(&event));
                },
            }
        }
    }

    pub fn local_partition(&mut self) -> &mut P {
        self.local_partition
    }
}
