#[cfg(not(target_os = "cuda"))]
use core::iter::Iterator;

use rust_cuda::{rustacuda_core::DeviceCopy, utils::exchange::buffer::CudaExchangeBuffer};

#[cfg(not(target_os = "cuda"))]
use rust_cuda::rustacuda::{
    error::CudaResult,
    function::{BlockSize, GridSize},
};

use super::utils::MaybeSome;

#[derive(rust_cuda::common::RustToCudaAsRust, rust_cuda::common::LendRustBorrowToCuda)]
#[allow(clippy::module_name_repetitions)]
pub struct ValueBuffer<T: Clone + DeviceCopy> {
    #[r2cEmbed]
    mask: CudaExchangeBuffer<bool>,
    #[r2cEmbed]
    buffer: CudaExchangeBuffer<MaybeSome<T>>,
}

#[cfg(not(target_os = "cuda"))]
impl<T: Clone + DeviceCopy> ValueBuffer<T> {
    /// # Errors
    /// Returns a `rustacuda::errors::CudaError` iff an error occurs inside CUDA
    pub fn new(block_size: &BlockSize, grid_size: &GridSize) -> CudaResult<Self> {
        let block_size = (block_size.x * block_size.y * block_size.z) as usize;
        let grid_size = (grid_size.x * grid_size.y * grid_size.z) as usize;
        let total_capacity = block_size * grid_size;

        let mut buffer = alloc::vec::Vec::with_capacity(total_capacity);
        buffer.resize_with(total_capacity, || MaybeSome::None);

        Ok(Self {
            mask: CudaExchangeBuffer::new(&false, total_capacity)?,
            buffer: CudaExchangeBuffer::from_vec(buffer)?,
        })
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.buffer.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = Option<&T>> {
        self.mask
            .iter()
            .zip(self.buffer.iter())
            .map(|(mask, maybe)| {
                if *mask {
                    Some(unsafe { maybe.assume_some_ref() })
                } else {
                    None
                }
            })
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = ValueRefMut<T>> {
        self.mask
            .iter_mut()
            .zip(self.buffer.iter_mut())
            .map(|(mask, value)| ValueRefMut { mask, value })
    }
}

#[cfg(target_os = "cuda")]
impl<T: Clone + DeviceCopy> ValueBuffer<T> {
    pub fn with_value_for_core<F: FnOnce(Option<T>) -> Option<T>>(&mut self, inner: F) {
        let index = rust_cuda::device::utils::index();

        // TODO: Check no spill to local memory
        let value = if self.mask.get(index).copied().unwrap_or(false) {
            Some(unsafe { self.buffer.get_unchecked(index).assume_some_read() })
        } else {
            None
        };

        let result = inner(value);

        if let Some(mask) = self.mask.get_mut(index) {
            *mask = result.is_some();

            if let Some(result) = result {
                *unsafe { self.buffer.get_unchecked_mut(index) } = MaybeSome::Some(result);
            }
        }
    }
}

pub struct ValueRefMut<'v, T: DeviceCopy> {
    mask: &'v mut bool,
    value: &'v mut MaybeSome<T>,
}

impl<'v, T: DeviceCopy> ValueRefMut<'v, T> {
    pub fn take(&mut self) -> Option<T> {
        if *self.mask {
            *self.mask = false;

            Some(unsafe { self.value.assume_some_read() })
        } else {
            None
        }
    }

    #[must_use]
    pub fn as_ref(&self) -> Option<&T> {
        if *self.mask {
            Some(unsafe { self.value.assume_some_ref() })
        } else {
            None
        }
    }

    #[must_use]
    pub fn as_mut(&mut self) -> Option<&mut T> {
        if *self.mask {
            Some(unsafe { self.value.assume_some_mut() })
        } else {
            None
        }
    }

    pub fn replace(&mut self, value: Option<T>) -> Option<T> {
        let old = if *self.mask {
            Some(unsafe { self.value.assume_some_read() })
        } else {
            None
        };

        *self.mask = value.is_some();

        if let Some(value) = value {
            *self.value = MaybeSome::Some(value);
        }

        old
    }
}
