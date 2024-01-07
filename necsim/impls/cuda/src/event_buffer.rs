use core::fmt;

#[cfg(not(target_os = "cuda"))]
use rust_cuda::deps::rustacuda::{
    error::CudaResult,
    function::{BlockSize, GridSize},
};

use rust_cuda::utils::{
    aliasing::SplitSliceOverCudaThreadsDynamicStride, exchange::buffer::CudaExchangeBuffer,
};

use necsim_core::{
    event::{PackedEvent, SpeciationEvent, TypedEvent},
    reporter::{
        boolean::{Boolean, False, True},
        Reporter,
    },
};

#[cfg(target_os = "cuda")]
use necsim_core::impl_report;

use super::utils::MaybeSome;

#[allow(clippy::module_name_repetitions, clippy::type_complexity)]
#[derive(rust_cuda::lend::LendRustToCuda)]
#[cuda(free = "ReportSpeciation", free = "ReportDispersal")]
pub struct EventBuffer<ReportSpeciation: Boolean, ReportDispersal: Boolean> {
    #[cuda(embed)]
    event_mask: SplitSliceOverCudaThreadsDynamicStride<CudaExchangeBuffer<bool, true, true>>,
    #[cuda(embed)]
    event_buffer: SplitSliceOverCudaThreadsDynamicStride<
        CudaExchangeBuffer<
            MaybeSome<<EventBuffer<ReportSpeciation, ReportDispersal> as EventType>::Event>,
            false,
            true,
        >,
    >,
    max_events: usize,
    event_counter: usize,
}

pub trait EventType {
    type Event: 'static
        + Sync
        + rust_cuda::deps::const_type_layout::TypeGraphLayout
        + rust_cuda::safety::StackOnly
        + rust_cuda::safety::PortableBitSemantics
        + Into<TypedEvent>
        + Into<PackedEvent>
        + Clone;
}

impl<ReportSpeciation: Boolean, ReportDispersal: Boolean> EventType
    for EventBuffer<ReportSpeciation, ReportDispersal>
{
    default type Event = PackedEvent;
}

impl EventType for EventBuffer<False, False> {
    type Event = PackedEvent;
}

impl EventType for EventBuffer<False, True> {
    type Event = PackedEvent;
}

impl EventType for EventBuffer<True, False> {
    type Event = SpeciationEvent;
}

impl EventType for EventBuffer<True, True> {
    type Event = PackedEvent;
}

impl<ReportSpeciation: Boolean, ReportDispersal: Boolean> fmt::Debug
    for EventBuffer<ReportSpeciation, ReportDispersal>
{
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_struct("EventBuffer")
            .field("max_events", &self.max_events)
            .field("event_counter", &self.event_counter)
            .finish_non_exhaustive()
    }
}

#[cfg(not(target_os = "cuda"))]
impl<ReportSpeciation: Boolean, ReportDispersal: Boolean>
    EventBuffer<ReportSpeciation, ReportDispersal>
{
    /// # Errors
    /// Returns a `rustacuda::errors::CudaError` iff an error occurs inside CUDA
    pub fn new(
        block_size: &BlockSize,
        grid_size: &GridSize,
        max_events: usize,
    ) -> CudaResult<Self> {
        let block_size = (block_size.x * block_size.y * block_size.z) as usize;
        let grid_size = (grid_size.x * grid_size.y * grid_size.z) as usize;

        #[allow(clippy::bool_to_int_with_if)]
        let max_events = if ReportDispersal::VALUE {
            max_events
        } else if ReportSpeciation::VALUE {
            1_usize
        } else {
            0_usize
        };

        let event_capacity = max_events * block_size * grid_size;

        let mut event_buffer = alloc::vec::Vec::with_capacity(event_capacity);
        event_buffer.resize_with(event_capacity, || MaybeSome::None);

        Ok(Self {
            event_mask: SplitSliceOverCudaThreadsDynamicStride::new(
                CudaExchangeBuffer::new(&false, event_capacity)?,
                max_events,
            ),
            event_buffer: SplitSliceOverCudaThreadsDynamicStride::new(
                CudaExchangeBuffer::from_vec(event_buffer)?,
                max_events,
            ),
            max_events,
            event_counter: 0_usize,
        })
    }

    pub fn report_events_unordered<P>(&mut self, reporter: &mut P)
    where
        P: Reporter<ReportSpeciation = ReportSpeciation, ReportDispersal = ReportDispersal>,
    {
        for (mask, event) in self.event_mask.iter_mut().zip(self.event_buffer.iter()) {
            if *mask.read() {
                let event: TypedEvent = unsafe { event.read().assume_some_read() }.into();

                match event {
                    TypedEvent::Speciation(ref speciation) => {
                        reporter.report_speciation(speciation.into());
                    },
                    TypedEvent::Dispersal(ref dispersal) => {
                        reporter.report_dispersal(dispersal.into());
                    },
                }
            }

            mask.write(false);
        }
    }

    pub fn max_events_per_individual(&self) -> usize {
        self.max_events
    }
}

#[cfg(target_os = "cuda")]
impl<ReportSpeciation: Boolean, ReportDispersal: Boolean> Reporter
    for EventBuffer<ReportSpeciation, ReportDispersal>
{
    impl_report!([default] speciation(&mut self, _event: Ignored) {});

    impl_report!([default] dispersal(&mut self, _event: Ignored) {});

    impl_report!([default] progress(&mut self, _progress: Ignored) {});
}

#[cfg(target_os = "cuda")]
impl Reporter for EventBuffer<False, True> {
    impl_report!(
        #[debug_requires(
            self.event_counter < self.max_events,
            "does not report extraneous dispersal events"
        )]
        dispersal(&mut self, event: Used) {
            if let Some(mask) = self.event_mask.get_mut(self.event_counter) {
                mask.write(true);

                unsafe {
                    self.event_buffer.get_unchecked_mut(self.event_counter)
                }.write(MaybeSome::Some(event.clone().into()));
            }

            self.event_counter += 1;
        }
    );
}

#[cfg(target_os = "cuda")]
impl Reporter for EventBuffer<True, False> {
    impl_report!(
        #[debug_requires(
            self.event_counter == 0,
            "does not report extraneous speciation events"
        )]
        speciation(&mut self, event: Used) {
            if let Some(mask) = self.event_mask.get_mut(0) {
                mask.write(true);

                unsafe {
                    self.event_buffer.get_unchecked_mut(0)
                }.write(MaybeSome::Some(event.clone()));
            }

            self.event_counter = self.max_events;
        }
    );
}

#[cfg(target_os = "cuda")]
impl Reporter for EventBuffer<True, True> {
    impl_report!(
        #[debug_requires(
            self.event_counter < self.max_events,
            "does not report extraneous speciation events"
        )]
        speciation(&mut self, event: Used) {
            if let Some(mask) = self.event_mask.get_mut(self.event_counter) {
                mask.write(true);

                unsafe {
                    self.event_buffer.get_unchecked_mut(self.event_counter)
                }.write(MaybeSome::Some(event.clone().into()));
            }

            self.event_counter = self.max_events;
        }
    );

    impl_report!(
        #[debug_requires(
            self.event_counter < self.max_events,
            "does not report extraneous dispersal events"
        )]
        dispersal(&mut self, event: Used) {
            if let Some(mask) = self.event_mask.get_mut(self.event_counter) {
                mask.write(true);

                unsafe {
                    self.event_buffer.get_unchecked_mut(self.event_counter)
                }.write(MaybeSome::Some(event.clone().into()));
            }

            self.event_counter += 1;
        }
    );
}
