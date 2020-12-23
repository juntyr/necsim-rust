use core::ops::DerefMut;

use rustacuda::{
    error::CudaResult,
    function::{BlockSize, GridSize},
    memory::{CopyDestination, DeviceBox, DeviceBuffer, LockedBuffer},
};

use rustacuda_core::DeviceCopy;

use rust_cuda::{common::DeviceBoxMut, host::CudaDropWrapper};

#[allow(clippy::module_name_repetitions)]
pub struct ValueBufferHost<T: Clone + DeviceCopy> {
    host_buffer: CudaDropWrapper<LockedBuffer<Option<T>>>,
    device_buffer: CudaDropWrapper<DeviceBuffer<Option<T>>>,
    cuda_repr_box: CudaDropWrapper<DeviceBox<super::common::ValueBufferCudaRepresentation<T>>>,
}

impl<T: Clone + DeviceCopy> ValueBufferHost<T> {
    /// # Errors
    /// Returns a `rustacuda::errors::CudaError` iff an error occurs inside CUDA
    pub fn new(block_size: &BlockSize, grid_size: &GridSize) -> CudaResult<Self> {
        let block_size = (block_size.x * block_size.y * block_size.z) as usize;
        let grid_size = (grid_size.x * grid_size.y * grid_size.z) as usize;
        let total_capacity = block_size * grid_size;

        let host_buffer = CudaDropWrapper::from(LockedBuffer::new(&None, total_capacity)?);
        let mut device_buffer =
            CudaDropWrapper::from(DeviceBuffer::from_slice(host_buffer.as_slice())?);

        let cuda_repr = super::common::ValueBufferCudaRepresentation {
            value_buffer: (device_buffer.as_device_ptr(), device_buffer.len()),
        };

        let cuda_repr_box = CudaDropWrapper::from(DeviceBox::new(&cuda_repr)?);

        Ok(Self {
            host_buffer,
            device_buffer,
            cuda_repr_box,
        })
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.host_buffer.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.host_buffer.is_empty()
    }

    /// # Errors
    /// Returns a `rustacuda::errors::CudaError` iff an error occurs inside CUDA
    pub fn with_upload_and_fetch_values<
        A,
        Q,
        U: FnOnce(&mut A, &mut [Option<T>]),
        I: FnOnce(DeviceBoxMut<super::common::ValueBufferCudaRepresentation<T>>) -> CudaResult<Q>,
        F: FnOnce(&mut A, &mut [Option<T>]),
    >(
        &mut self,
        auxiliary: &mut A,
        upload: U,
        inner: I,
        fetch: F,
    ) -> CudaResult<Q> {
        upload(auxiliary, self.host_buffer.as_mut_slice());

        self.device_buffer.copy_from(self.host_buffer.deref_mut())?;

        let result = inner(DeviceBoxMut::from(&mut self.cuda_repr_box))?;

        self.device_buffer.copy_to(self.host_buffer.deref_mut())?;

        fetch(auxiliary, self.host_buffer.as_mut_slice());

        Ok(result)
    }
}
