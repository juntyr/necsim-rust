use core::fmt;

#[cfg(not(target_os = "cuda"))]
use rust_cuda::rustacuda::{
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
#[derive(rust_cuda::common::LendRustToCuda)]
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
        + ~const rust_cuda::const_type_layout::TypeGraphLayout
        + rust_cuda::safety::StackOnly
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
            .finish()
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
        if ReportDispersal::VALUE {
            for (mask, dispersal) in self
                .dispersal_mask
                .iter_mut()
                .zip(self.dispersal_buffer.iter())
            {
                if *mask.read() {
                    reporter.report_dispersal(unsafe { dispersal.read().assume_some_ref() }.into());
                }

                mask.write(false);
            }
        }

        if ReportSpeciation::VALUE {
            for (mask, speciation) in self
                .speciation_mask
                .iter_mut()
                .zip(self.speciation_buffer.iter())
            {
                if *mask.read() {
                    reporter
                        .report_speciation(unsafe { speciation.read().assume_some_ref() }.into());
                }

                mask.write(false);
            }
        }
    }
}

#[cfg(not(target_os = "cuda"))]
impl<ReportSpeciation: Boolean, ReportDispersal: Boolean>
    EventBuffer<ReportSpeciation, ReportDispersal>
{
    pub fn sort_events(&mut self) {
        if ReportDispersal::VALUE {
            let mut events: alloc::vec::Vec<DispersalEvent> = alloc::vec::Vec::new();

            for (mask, dispersal) in self
                .dispersal_mask
                .iter_mut()
                .zip(self.dispersal_buffer.iter())
            {
                if *mask.read() {
                    events.push(unsafe { dispersal.read().assume_some_read() });
                }

                mask.write(false);
            }

            events.sort_unstable();

            for ((event, mask), dispersal) in events
                .into_iter()
                .zip(self.dispersal_mask.iter_mut())
                .zip(self.dispersal_buffer.iter_mut())
            {
                *dispersal.as_scratch_mut() = MaybeSome::Some(event);
                mask.write(true);
            }
        }

        if ReportSpeciation::VALUE {
            let mut events: alloc::vec::Vec<SpeciationEvent> = alloc::vec::Vec::new();

            for (mask, speciation) in self
                .speciation_mask
                .iter_mut()
                .zip(self.speciation_buffer.iter())
            {
                if *mask.read() {
                    events.push(unsafe { speciation.read().assume_some_read() });
                }

                mask.write(false);
            }

            events.sort_unstable();

            for ((event, mask), speciation) in events
                .into_iter()
                .zip(self.speciation_mask.iter_mut())
                .zip(self.speciation_buffer.iter_mut())
            {
                *speciation.as_scratch_mut() = MaybeSome::Some(event);
                mask.write(true);
            }
        }
    }
}

#[cfg(target_os = "cuda")]
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SortStepDirection {
    Less,
    Greater,
}

#[cfg(target_os = "cuda")]
impl<ReportSpeciation: Boolean, ReportDispersal: Boolean>
    EventBuffer<ReportSpeciation, ReportDispersal>
{
    #[allow(clippy::too_many_lines)]
    /// # Safety
    ///
    /// All CUDA threads must call this method with the same size, stride, and
    /// direction arguments. Only one call per kernel launch is safe without
    /// further synchronisation.
    pub unsafe fn sort_events_step(
        &mut self,
        size: usize,
        stride: usize,
        direction: SortStepDirection,
    ) {
        use core::cmp::Ordering;

        if ReportDispersal::VALUE {
            let idx = rust_cuda::device::utils::index();

            // odd-even merge position
            let pos = 2 * idx - (idx & (stride - 1));

            let (pos_a, pos_b) = if stride < (size / 2) {
                (pos - stride, pos)
            } else {
                (pos, pos + stride)
            };

            let offset = idx & ((size / 2) - 1);

            if (stride >= (size / 2)) || (offset >= stride) {
                let mask_a: bool = *self.dispersal_mask.alias_unchecked()[pos_a].read();
                let mask_b: bool = *self.dispersal_mask.alias_unchecked()[pos_b].read();

                let cmp = match (mask_a, mask_b) {
                    (false, false) => Ordering::Equal,
                    (false, true) => Ordering::Greater,
                    (true, false) => Ordering::Less,
                    (true, true) => {
                        // Safety: both masks indicate that the two events exist
                        let event_a: &DispersalEvent = unsafe {
                            self.dispersal_buffer.alias_unchecked()[pos_a]
                                .as_uninit()
                                .assume_init_ref()
                                .assume_some_ref()
                        };
                        let event_b: &DispersalEvent = unsafe {
                            self.dispersal_buffer.alias_unchecked()[pos_b]
                                .as_uninit()
                                .assume_init_ref()
                                .assume_some_ref()
                        };

                        event_a.cmp(event_b)
                    },
                };

                if let (SortStepDirection::Greater, Ordering::Greater)
                | (SortStepDirection::Less, Ordering::Less) = (direction, cmp)
                {
                    self.dispersal_mask.alias_mut_unchecked()[pos_a].write(mask_b);
                    self.dispersal_mask.alias_mut_unchecked()[pos_b].write(mask_a);

                    match (mask_a, mask_b) {
                        (false, false) => (),
                        (false, true) => {
                            let event_b: DispersalEvent = unsafe {
                                self.dispersal_buffer.alias_unchecked()[pos_b]
                                    .as_uninit()
                                    .assume_init_ref()
                                    .assume_some_read()
                            };

                            unsafe {
                                self.dispersal_buffer.alias_mut_unchecked()[pos_a]
                                    .write(MaybeSome::Some(event_b));
                            }
                        },
                        (true, false) => {
                            let event_a: DispersalEvent = unsafe {
                                self.dispersal_buffer.alias_unchecked()[pos_a]
                                    .as_uninit()
                                    .assume_init_ref()
                                    .assume_some_read()
                            };

                            unsafe {
                                self.dispersal_buffer.alias_mut_unchecked()[pos_b]
                                    .write(MaybeSome::Some(event_a));
                            }
                        },
                        (true, true) => {
                            let event_a: DispersalEvent = unsafe {
                                self.dispersal_buffer.alias_unchecked()[pos_a]
                                    .as_uninit()
                                    .assume_init_ref()
                                    .assume_some_read()
                            };
                            let event_b: DispersalEvent = unsafe {
                                self.dispersal_buffer.alias_unchecked()[pos_b]
                                    .as_uninit()
                                    .assume_init_ref()
                                    .assume_some_read()
                            };

                            unsafe {
                                self.dispersal_buffer.alias_mut_unchecked()[pos_a]
                                    .write(MaybeSome::Some(event_b));
                                self.dispersal_buffer.alias_mut_unchecked()[pos_b]
                                    .write(MaybeSome::Some(event_a));
                            }
                        },
                    }
                }
            }
        }

        if ReportSpeciation::VALUE {
            let idx = rust_cuda::device::utils::index();

            // odd-even merge position
            let pos = 2 * idx - (idx & (stride - 1));

            let (pos_a, pos_b) = if stride < (size / 2) {
                (pos - stride, pos)
            } else {
                (pos, pos + stride)
            };

            let offset = idx & ((size / 2) - 1);

            if (stride >= (size / 2)) || (offset >= stride) {
                let mask_a: bool = *self.speciation_mask.alias_unchecked()[pos_a].read();
                let mask_b: bool = *self.speciation_mask.alias_unchecked()[pos_b].read();

                let cmp = match (mask_a, mask_b) {
                    (false, false) => Ordering::Equal,
                    (false, true) => Ordering::Greater,
                    (true, false) => Ordering::Less,
                    (true, true) => {
                        // Safety: both masks indicate that the two events exist
                        let event_a: &SpeciationEvent = unsafe {
                            self.speciation_buffer.alias_unchecked()[pos_a]
                                .as_uninit()
                                .assume_init_ref()
                                .assume_some_ref()
                        };
                        let event_b: &SpeciationEvent = unsafe {
                            self.speciation_buffer.alias_unchecked()[pos_b]
                                .as_uninit()
                                .assume_init_ref()
                                .assume_some_ref()
                        };

                        event_a.cmp(event_b)
                    },
                };

                if let (SortStepDirection::Greater, Ordering::Greater)
                | (SortStepDirection::Less, Ordering::Less) = (direction, cmp)
                {
                    self.speciation_mask.alias_mut_unchecked()[pos_a].write(mask_b);
                    self.speciation_mask.alias_mut_unchecked()[pos_b].write(mask_a);

                    match (mask_a, mask_b) {
                        (false, false) => (),
                        (false, true) => {
                            let event_b: SpeciationEvent = unsafe {
                                self.speciation_buffer.alias_unchecked()[pos_b]
                                    .as_uninit()
                                    .assume_init_ref()
                                    .assume_some_read()
                            };

                            unsafe {
                                self.speciation_buffer.alias_mut_unchecked()[pos_a]
                                    .write(MaybeSome::Some(event_b));
                            }
                        },
                        (true, false) => {
                            let event_a: SpeciationEvent = unsafe {
                                self.speciation_buffer.alias_unchecked()[pos_a]
                                    .as_uninit()
                                    .assume_init_ref()
                                    .assume_some_read()
                            };

                            unsafe {
                                self.speciation_buffer.alias_mut_unchecked()[pos_b]
                                    .write(MaybeSome::Some(event_a));
                            }
                        },
                        (true, true) => {
                            let event_a: SpeciationEvent = unsafe {
                                self.speciation_buffer.alias_unchecked()[pos_a]
                                    .as_uninit()
                                    .assume_init_ref()
                                    .assume_some_read()
                            };
                            let event_b: SpeciationEvent = unsafe {
                                self.speciation_buffer.alias_unchecked()[pos_b]
                                    .as_uninit()
                                    .assume_init_ref()
                                    .assume_some_read()
                            };

                            unsafe {
                                self.speciation_buffer.alias_mut_unchecked()[pos_a]
                                    .write(MaybeSome::Some(event_b));
                                self.speciation_buffer.alias_mut_unchecked()[pos_b]
                                    .write(MaybeSome::Some(event_a));
                            }
                        },
                    }
                }
            }
        }
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
