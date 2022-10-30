use core::{fmt, marker::PhantomData};

#[cfg(not(target_os = "cuda"))]
use rust_cuda::rustacuda::{
    error::CudaResult,
    function::{BlockSize, GridSize},
};

use rust_cuda::utils::{
    aliasing::{SplitSliceOverCudaThreadsConstStride, SplitSliceOverCudaThreadsDynamicStride},
    exchange::buffer::CudaExchangeBuffer,
};

use necsim_core::{
    event::{DispersalEvent, SpeciationEvent},
    reporter::{boolean::Boolean, Reporter},
};

#[cfg(target_os = "cuda")]
use necsim_core::impl_report;

use super::utils::MaybeSome;

#[allow(clippy::module_name_repetitions)]
#[derive(rust_cuda::common::LendRustToCuda)]
#[r2cLayout(free = "ReportSpeciation")]
#[r2cLayout(free = "ReportDispersal")]
pub struct EventBuffer<ReportSpeciation: Boolean, ReportDispersal: Boolean> {
    #[r2cEmbed]
    speciation_mask:
        SplitSliceOverCudaThreadsConstStride<CudaExchangeBuffer<bool, true, true>, 1_usize>,
    #[r2cEmbed]
    speciation_buffer: SplitSliceOverCudaThreadsConstStride<
        CudaExchangeBuffer<MaybeSome<SpeciationEvent>, false, true>,
        1_usize,
    >,
    #[r2cEmbed]
    dispersal_mask: SplitSliceOverCudaThreadsDynamicStride<CudaExchangeBuffer<bool, true, true>>,
    #[r2cEmbed]
    dispersal_buffer: SplitSliceOverCudaThreadsDynamicStride<
        CudaExchangeBuffer<MaybeSome<DispersalEvent>, false, true>,
    >,
    max_events: usize,
    event_counter: usize,
    marker: PhantomData<(ReportSpeciation, ReportDispersal)>,
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

        let speciation_capacity = if ReportSpeciation::VALUE {
            block_size * grid_size
        } else {
            0_usize
        };
        let dispersal_capacity = if ReportDispersal::VALUE {
            max_events * block_size * grid_size
        } else {
            0_usize
        };

        let mut speciation_buffer = alloc::vec::Vec::with_capacity(speciation_capacity);
        speciation_buffer.resize_with(speciation_capacity, || MaybeSome::None);

        let mut dispersal_buffer = alloc::vec::Vec::with_capacity(dispersal_capacity);
        dispersal_buffer.resize_with(dispersal_capacity, || MaybeSome::None);

        Ok(Self {
            speciation_mask: SplitSliceOverCudaThreadsConstStride::new(CudaExchangeBuffer::new(
                &false,
                speciation_capacity,
            )?),
            speciation_buffer: SplitSliceOverCudaThreadsConstStride::new(
                CudaExchangeBuffer::from_vec(speciation_buffer)?,
            ),
            dispersal_mask: SplitSliceOverCudaThreadsDynamicStride::new(
                CudaExchangeBuffer::new(&false, dispersal_capacity)?,
                max_events,
            ),
            dispersal_buffer: SplitSliceOverCudaThreadsDynamicStride::new(
                CudaExchangeBuffer::from_vec(dispersal_buffer)?,
                max_events,
            ),
            max_events,
            event_counter: 0_usize,
            marker: PhantomData::<(ReportSpeciation, ReportDispersal)>,
        })
    }

    pub fn report_events<P>(&mut self, reporter: &mut P)
    where
        P: Reporter<ReportSpeciation = ReportSpeciation, ReportDispersal = ReportDispersal>,
    {
        for (mask, dispersal) in self
            .dispersal_mask
            .iter_mut()
            .zip(self.dispersal_buffer.iter())
        {
            if ReportDispersal::VALUE && *mask.read() {
                reporter.report_dispersal(unsafe { dispersal.read().assume_some_ref() }.into());
            }

            mask.write(false);
        }

        for (mask, speciation) in self
            .speciation_mask
            .iter_mut()
            .zip(self.speciation_buffer.iter())
        {
            if ReportSpeciation::VALUE && *mask.read() {
                reporter.report_speciation(unsafe { speciation.read().assume_some_ref() }.into());
            }

            mask.write(false);
        }
    }
}

#[cfg(target_os = "cuda")]
impl<ReportSpeciation: Boolean, ReportDispersal: Boolean> Reporter
    for EventBuffer<ReportSpeciation, ReportDispersal>
{
    impl_report!(
        #[debug_requires(
            !self.speciation_mask.get(0).map(
                rust_cuda::utils::exchange::buffer::CudaExchangeItem::read
            ).copied().unwrap_or(true),
            "does not report extraneous speciation event"
        )]
        speciation(&mut self, event: Used) {
            if ReportSpeciation::VALUE {
                if let Some(mask) = self.speciation_mask.get_mut(0) {
                    mask.write(true);

                    unsafe {
                        self.speciation_buffer.get_unchecked_mut(0)
                    }.write(MaybeSome::Some(event.clone()));
                }
            }
        }
    );

    impl_report!(
        #[debug_requires(
            self.event_counter < self.max_events,
            "does not report extraneous dispersal events"
        )]
        dispersal(&mut self, event: Used) {
            if ReportDispersal::VALUE {
                if let Some(mask) = self.dispersal_mask.get_mut(self.event_counter) {
                    mask.write(true);

                    unsafe {
                        self.dispersal_buffer.get_unchecked_mut(self.event_counter)
                    }.write(MaybeSome::Some(event.clone()));
                }

                self.event_counter += 1;
            }
        }
    );

    impl_report!(progress(&mut self, _progress: Ignored) {});
}
