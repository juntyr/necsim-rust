use core::ops::{Deref, DerefMut};

use rust_cuda::utils::exchange::buffer::CudaExchangeBuffer;
use rustacuda_core::DeviceCopy;

#[derive(RustToCuda, LendToCuda)]
#[allow(clippy::module_name_repetitions)]
pub struct ValueBuffer<T: Clone + DeviceCopy> {
    #[r2cEmbed]
    buffer: CudaExchangeBuffer<Option<T>>,
}

#[cfg(not(target_os = "cuda"))]
use rustacuda::{
    error::CudaResult,
    function::{BlockSize, GridSize},
};

impl<T: Clone + DeviceCopy> Deref for ValueBuffer<T> {
    type Target = [Option<T>];

    fn deref(&self) -> &Self::Target {
        &self.buffer
    }
}

impl<T: Clone + DeviceCopy> DerefMut for ValueBuffer<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.buffer
    }
}

#[cfg(not(target_os = "cuda"))]
impl<T: Clone + DeviceCopy> ValueBuffer<T> {
    /// # Errors
    /// Returns a `rustacuda::errors::CudaError` iff an error occurs inside CUDA
    pub fn new(block_size: &BlockSize, grid_size: &GridSize) -> CudaResult<Self> {
        let block_size = (block_size.x * block_size.y * block_size.z) as usize;
        let grid_size = (grid_size.x * grid_size.y * grid_size.z) as usize;
        let total_capacity = block_size * grid_size;

        Ok(Self {
            buffer: CudaExchangeBuffer::new(&None, total_capacity)?,
        })
    }
}

#[cfg(target_os = "cuda")]
impl<T: Clone + DeviceCopy> ValueBuffer<T> {
    pub fn with_value_for_core<F: FnOnce(Option<T>) -> Option<T>>(&mut self, inner: F) {
        let index = rust_cuda::device::utils::index();

        // `take()` would be semantically better - but `clone()` does not spill to local
        // memory
        let value = self.buffer[index].clone();

        self.buffer[index] = inner(value);
    }
}
