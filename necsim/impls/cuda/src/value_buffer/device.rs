use alloc::boxed::Box;

use rustacuda_core::DeviceCopy;

use rust_cuda::common::DeviceBoxMut;

#[allow(clippy::module_name_repetitions)]
pub struct ValueBufferDevice<T: Clone + DeviceCopy> {
    value_buffer: Box<[Option<T>]>,
}

impl<T: Clone + DeviceCopy> ValueBufferDevice<T> {
    /// # Safety
    /// This function is only safe to call iff `cuda_repr_mut` is the
    /// `DeviceBoxMut` borrowed on the CPU using the corresponding
    /// `TaskListHost::get_mut_cuda_ptr`.
    pub unsafe fn with_borrow_from_rust_mut<O, F: FnOnce(&mut Self) -> O>(
        mut cuda_repr_mut: DeviceBoxMut<super::common::ValueBufferCudaRepresentation<T>>,
        inner: F,
    ) -> O {
        let cuda_repr_ref = cuda_repr_mut.as_mut();

        let raw_slice: &mut [Option<T>] = core::slice::from_raw_parts_mut(
            cuda_repr_ref.value_buffer.0.as_raw_mut(),
            cuda_repr_ref.value_buffer.1,
        );

        let mut rust_repr = ValueBufferDevice {
            value_buffer: alloc::boxed::Box::from_raw(raw_slice),
        };

        let result = inner(&mut rust_repr);

        // MUST forget about rust_repr as we do NOT own any of the heap memory
        // it might reference
        core::mem::forget(rust_repr);

        result
    }

    pub fn with_value_for_core<F: FnOnce(Option<T>) -> Option<T>>(&mut self, inner: F) {
        let index = rust_cuda::device::utils::index();

        let task = self.value_buffer[index].take();

        self.value_buffer[index] = inner(task);
    }
}
