use core::marker::PhantomData;

use rustacuda_core::{DeviceCopy, DevicePointer};

use rust_cuda::common::RustToCuda;

use necsim_core::cogs::{Habitat, LineageReference};

#[allow(clippy::module_name_repetitions)]
#[allow(dead_code)]
pub struct TaskListCudaRepresentation<H: Habitat + RustToCuda, R: LineageReference<H> + DeviceCopy>
{
    pub(super) task_list: (DevicePointer<Option<R>>, usize),
    pub(super) marker: PhantomData<(H, R)>,
}

unsafe impl<H: Habitat + RustToCuda, R: LineageReference<H> + DeviceCopy> DeviceCopy
    for TaskListCudaRepresentation<H, R>
{
}
