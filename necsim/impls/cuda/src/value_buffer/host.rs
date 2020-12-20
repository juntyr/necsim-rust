use core::ops::DerefMut;

use rustacuda::{
    error::CudaResult,
    function::{BlockSize, GridSize},
    memory::{CopyDestination, DeviceBox, DeviceBuffer, LockedBuffer},
};

use rustacuda_core::{DeviceCopy, DevicePointer};

use rust_cuda::host::CudaDropWrapper;

// Should be used as member for RustToCuda deriving struct
use core::ops::Deref;
struct CudaInterchangeBufferHost<T: Clone + DeviceCopy> {
    host_buffer: CudaDropWrapper<LockedBuffer<T>>,
    device_buffer: CudaDropWrapper<DeviceBuffer<T>>,
}
impl<T: Clone + DeviceCopy> Deref for CudaInterchangeBufferHost<T> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        self.host_buffer.as_slice()
    }
}
impl<T: Clone + DeviceCopy> DerefMut for CudaInterchangeBufferHost<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.host_buffer.as_mut_slice()
    }
}

// Should be constructed for RustToCuda Cuda representation
struct CudaInterchangeBufferIntermediate<T: Clone + DeviceCopy>(DevicePointer<T>, usize);

// Should be constructed automatically from Cuda representation -> to avoid
// name changes we could simply perform some switcheroo of struct definitions
// for different targets
struct CudaInterchangeBufferDevice<T: Clone + DeviceCopy>(
    core::mem::ManuallyDrop<alloc::boxed::Box<[T]>>,
);
impl<T: Clone + DeviceCopy> Deref for CudaInterchangeBufferDevice<T> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl<T: Clone + DeviceCopy> DerefMut for CudaInterchangeBufferDevice<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

// Need CudaInterchange trait / wrapper that only allocates up front:
// 1) full access to CPU structure + make changes
// 2) transform into structural equivalent that can be used to send to CUDA ->
// copy to device 3) transform back into CPU structure -> copy from device
// There should also be the ability to chain multiple together (depending on if
// I use closures or types)

pub struct ValueBuffer<T: Clone + DeviceCopy> {
    buffer: CudaInterchangeBufferHost<T>,
}

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
        I: FnOnce(DevicePointer<super::common::ValueBufferCudaRepresentation<T>>) -> CudaResult<Q>,
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

        let result = inner(self.cuda_repr_box.as_device_ptr())?;

        self.device_buffer.copy_to(self.host_buffer.deref_mut())?;

        fetch(auxiliary, self.host_buffer.as_mut_slice());

        Ok(result)
    }
}
