use core::{marker::PhantomData, ops::DerefMut};

use rustacuda::{
    error::CudaResult,
    function::{BlockSize, GridSize},
    memory::{CopyDestination, DeviceBox, DeviceBuffer, LockedBuffer},
};

use rustacuda_core::DeviceCopy;

use rust_cuda::common::{DeviceBoxMut, RustToCuda};

use rust_cuda::host::CudaDropWrapper;

use necsim_core::cogs::{Habitat, LineageReference};

#[allow(clippy::module_name_repetitions)]
pub struct TaskListHost<H: Habitat + RustToCuda, R: LineageReference<H> + DeviceCopy> {
    host_list: CudaDropWrapper<LockedBuffer<Option<R>>>,
    device_list: CudaDropWrapper<DeviceBuffer<Option<R>>>,
    cuda_repr_box: CudaDropWrapper<DeviceBox<super::common::TaskListCudaRepresentation<H, R>>>,
}

impl<H: Habitat + RustToCuda, R: LineageReference<H> + DeviceCopy> TaskListHost<H, R> {
    /// # Errors
    /// Returns a `rustacuda::errors::CudaError` iff an error occurs inside CUDA
    pub fn new(block_size: &BlockSize, grid_size: &GridSize) -> CudaResult<Self> {
        let block_size = (block_size.x * block_size.y * block_size.z) as usize;
        let grid_size = (grid_size.x * grid_size.y * grid_size.z) as usize;
        let total_capacity = block_size * grid_size;

        let host_list = CudaDropWrapper::from(LockedBuffer::new(&None, total_capacity)?);
        let mut device_list =
            CudaDropWrapper::from(DeviceBuffer::from_slice(host_list.as_slice())?);

        let cuda_repr = super::common::TaskListCudaRepresentation {
            task_list: (device_list.as_device_ptr(), device_list.len()),
            marker: PhantomData::<(H, R)>,
        };

        let cuda_repr_box = CudaDropWrapper::from(DeviceBox::new(&cuda_repr)?);

        Ok(Self {
            host_list,
            device_list,
            cuda_repr_box,
        })
    }

    /// # Errors
    /// Returns a `rustacuda::errors::CudaError` iff an error occurs inside CUDA
    pub fn with_upload_and_fetch_tasks<
        A,
        Q,
        U: FnOnce(&mut A, &mut [Option<R>]),
        I: FnOnce(
            &mut A,
            DeviceBoxMut<super::common::TaskListCudaRepresentation<H, R>>,
        ) -> CudaResult<Q>,
        F: FnOnce(&mut A, &mut [Option<R>]),
    >(
        &mut self,
        auxiliary: &mut A,
        upload: U,
        inner: I,
        fetch: F,
    ) -> CudaResult<Q> {
        upload(auxiliary, self.host_list.as_mut_slice());

        self.device_list.copy_from(self.host_list.deref_mut())?;

        let result = inner(auxiliary, DeviceBoxMut::from(&mut self.cuda_repr_box))?;

        self.device_list.copy_to(self.host_list.deref_mut())?;

        fetch(auxiliary, self.host_list.as_mut_slice());

        Ok(result)
    }
}
