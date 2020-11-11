use core::ops::DerefMut;

use rustacuda::error::CudaResult;
use rustacuda::function::{BlockSize, GridSize};
use rustacuda::memory::{CopyDestination, DeviceBox, DeviceBuffer, LockedBuffer};

use rustacuda_core::{DeviceCopy, DevicePointer};

use rust_cuda::common::RustToCuda;

use rust_cuda::host::CudaDropWrapper;

use necsim_core::cogs::{Habitat, LineageReference};
use necsim_core::event::Event;

#[allow(clippy::module_name_repetitions)]
pub struct EventBufferHost<
    H: Habitat + RustToCuda,
    R: LineageReference<H> + DeviceCopy,
    const REPORT_SPECIATION: bool,
    const REPORT_DISPERSAL: bool,
> {
    host_buffer: CudaDropWrapper<LockedBuffer<Option<Event<H, R>>>>,
    device_buffer: CudaDropWrapper<DeviceBuffer<Option<Event<H, R>>>>,
    cuda_repr_box: CudaDropWrapper<
        DeviceBox<
            super::common::EventBufferCudaRepresentation<H, R, REPORT_SPECIATION, REPORT_DISPERSAL>,
        >,
    >,
}

pub type EventIterator<'e, H, R> = core::iter::FilterMap<
    core::slice::IterMut<'e, Option<necsim_core::event::Event<H, R>>>,
    for<'r> fn(
        &'r mut Option<necsim_core::event::Event<H, R>>,
    ) -> Option<necsim_core::event::Event<H, R>>,
>;

impl<
        H: Habitat + RustToCuda,
        R: LineageReference<H> + DeviceCopy,
        const REPORT_SPECIATION: bool,
        const REPORT_DISPERSAL: bool,
    > EventBufferHost<H, R, REPORT_SPECIATION, REPORT_DISPERSAL>
{
    /// # Errors
    /// Returns a `rustacuda::errors::CudaError` iff an error occurs inside CUDA
    pub fn new(
        block_size: &BlockSize,
        grid_size: &GridSize,
        max_events: usize,
    ) -> CudaResult<Self> {
        let max_events = if REPORT_DISPERSAL {
            max_events
        } else if REPORT_SPECIATION {
            1_usize
        } else {
            0_usize
        };

        let block_size = (block_size.x * block_size.y * block_size.z) as usize;
        let grid_size = (grid_size.x * grid_size.y * grid_size.z) as usize;
        let total_capacity = max_events * block_size * grid_size;

        let host_buffer = CudaDropWrapper::from(LockedBuffer::new(&None, total_capacity)?);
        let mut device_buffer =
            CudaDropWrapper::from(DeviceBuffer::from_slice(host_buffer.as_slice())?);

        let cuda_repr = super::common::EventBufferCudaRepresentation {
            block_size,
            grid_size,
            max_events,
            device_buffer: device_buffer.as_device_ptr(),
        };

        let cuda_repr_box = CudaDropWrapper::from(DeviceBox::new(&cuda_repr)?);

        Ok(Self {
            host_buffer,
            device_buffer,
            cuda_repr_box,
        })
    }

    /// # Errors
    /// Returns a `rustacuda::errors::CudaError` iff an error occurs inside CUDA
    pub fn with_fetched_events<O, F: FnOnce(EventIterator<'_, H, R>) -> O>(
        &mut self,
        inner: F,
    ) -> CudaResult<O> {
        self.device_buffer.copy_to(self.host_buffer.deref_mut())?;

        // Collect the events and reset the buffer slice to all None's
        let result = inner(self.host_buffer.iter_mut().filter_map(Option::take));

        self.device_buffer.copy_from(self.host_buffer.deref_mut())?;

        Ok(result)
    }

    pub fn get_mut_cuda_ptr(
        &mut self,
    ) -> DevicePointer<
        super::common::EventBufferCudaRepresentation<H, R, REPORT_SPECIATION, REPORT_DISPERSAL>,
    > {
        self.cuda_repr_box.as_device_ptr()
    }
}
