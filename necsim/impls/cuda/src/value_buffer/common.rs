use rustacuda_core::{DeviceCopy, DevicePointer};

#[allow(clippy::module_name_repetitions)]
#[allow(dead_code)]
pub struct ValueBufferCudaRepresentation<T: Clone + DeviceCopy> {
    pub(super) value_buffer: (DevicePointer<Option<T>>, usize),
}

unsafe impl<T: Clone + DeviceCopy> DeviceCopy for ValueBufferCudaRepresentation<T> {}
