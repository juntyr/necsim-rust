use alloc::vec::Vec;

use rustacuda::error::CudaResult;
use rustacuda::function::{BlockSize, GridSize};
use rustacuda::memory::{CopyDestination, DeviceBox, DeviceBuffer, LockedBuffer};

use rustacuda_core::{DeviceCopy, DevicePointer};

use rust_cuda::common::RustToCuda;

use rust_cuda::host::CudaDropWrapper;

use necsim_core::cogs::{Habitat, LineageReference};
use necsim_core::event::Event;

#[allow(clippy::module_name_repetitions)]
pub struct EventBufferHost<H: Habitat + RustToCuda, R: LineageReference<H> + DeviceCopy> {
    block_size: usize,
    grid_size: usize,
    max_events: usize,
    host_buffer: CudaDropWrapper<LockedBuffer<Option<Event<H, R>>>>,
    device_buffer: CudaDropWrapper<DeviceBuffer<Option<Event<H, R>>>>,
    cuda_repr_box: CudaDropWrapper<DeviceBox<super::common::EventBufferCudaRepresentation<H, R>>>,
}

impl<H: Habitat + RustToCuda, R: LineageReference<H> + DeviceCopy> EventBufferHost<H, R> {
    /// # Errors
    /// Returns a `rustacuda::errors::CudaError` iff an error occurs inside CUDA
    pub fn new(
        block_size: &BlockSize,
        grid_size: &GridSize,
        max_events: usize,
    ) -> CudaResult<Self> {
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
            block_size,
            grid_size,
            max_events,
            host_buffer,
            device_buffer,
            cuda_repr_box,
        })
    }

    #[debug_requires(block_index < self.grid_size, "block_index is in range")]
    pub fn with_fetched_events_for_block<O, F: FnOnce(Vec<Event<H, R>>) -> O>(
        &mut self,
        block_index: usize,
        inner: F,
    ) -> CudaResult<O> {
        let full_host_buffer = self.host_buffer.as_mut_slice();
        let (_before_host_buffer, rest_host_buffer) =
            full_host_buffer.split_at_mut(block_index * self.block_size * self.max_events);
        let (block_host_buffer, _after_host_buffer) =
            rest_host_buffer.split_at_mut(self.block_size * self.max_events);

        let full_device_buffer = &mut self.device_buffer;
        let (_before_device_buffer, rest_device_buffer) =
            full_device_buffer.split_at_mut(block_index * self.block_size * self.max_events);
        let (block_device_buffer, _after_device_buffer) =
            rest_device_buffer.split_at_mut(self.block_size * self.max_events);

        block_device_buffer.copy_to(block_host_buffer)?;

        // Collect the events and reset the buffer slice to all None's
        let result = inner(
            block_host_buffer
                .iter_mut()
                .filter_map(Option::take)
                .collect::<Vec<Event<H, R>>>(),
        );

        block_device_buffer.copy_from(block_host_buffer)?;

        Ok(result)
    }

    pub fn get_mut_cuda_ptr(
        &mut self,
    ) -> DevicePointer<super::common::EventBufferCudaRepresentation<H, R>> {
        self.cuda_repr_box.as_device_ptr()
    }
}
