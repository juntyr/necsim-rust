use alloc::boxed::Box;
use core::marker::PhantomData;

use rustacuda_core::DeviceCopy;

use rust_cuda::common::RustToCuda;

use necsim_core::cogs::{Habitat, LineageReference};

#[allow(clippy::module_name_repetitions)]
pub struct TaskListDevice<H: Habitat + RustToCuda, R: LineageReference<H> + DeviceCopy> {
    task_list: Box<[Option<R>]>,
    marker: PhantomData<(H, R)>,
}

impl<H: Habitat + RustToCuda, R: LineageReference<H> + DeviceCopy> TaskListDevice<H, R> {
    /// # Safety
    /// This function is only safe to call iff `cuda_repr_ptr` is the
    /// `DevicePointer` borrowed on the CPU using the corresponding
    /// `TaskListHost::get_mut_cuda_ptr`.
    pub unsafe fn with_borrow_from_rust_mut<O, F: FnOnce(&mut Self) -> O>(
        cuda_repr_ptr: *mut super::common::TaskListCudaRepresentation<H, R>,
        inner: F,
    ) -> O {
        let cuda_repr_ref: &mut super::common::TaskListCudaRepresentation<H, R> =
            &mut *cuda_repr_ptr;

        let raw_slice: &mut [Option<R>] = core::slice::from_raw_parts_mut(
            cuda_repr_ref.task_list.0.as_raw_mut(),
            cuda_repr_ref.task_list.1,
        );

        let mut rust_repr = TaskListDevice {
            task_list: alloc::boxed::Box::from_raw(raw_slice),
            marker: PhantomData::<(H, R)>,
        };

        let result = inner(&mut rust_repr);

        // MUST forget about rust_repr as we do NOT own any of the heap memory
        // it might reference
        core::mem::forget(rust_repr);

        result
    }

    pub fn with_task_for_core<F: FnOnce(Option<R>) -> Option<R>>(&mut self, inner: F) {
        let index = rust_cuda::device::utils::index_no_offset();

        let task = self.task_list[index].take();

        self.task_list[index] = inner(task);
    }
}
