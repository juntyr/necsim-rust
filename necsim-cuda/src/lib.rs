#![deny(clippy::pedantic)]
#![no_std]
#![cfg(not(target_os = "cuda"))]

use core::ops::{Deref, DerefMut};

use rustacuda::error::{CudaError, CudaResult};
use rustacuda::memory::DeviceBuffer;

use rustacuda_core::DeviceCopy;

pub(crate) mod private {
    pub trait Sealed {}

    pub trait SealedCudaDrop: Sized {
        fn drop(val: Self) -> Result<(), (super::CudaError, Self)>;
    }
}

pub trait CudaAlloc: private::Sealed {}
impl<T: private::Sealed> CudaAlloc for T {}

pub struct NullCudaAlloc;
impl private::Sealed for NullCudaAlloc {}

pub struct ScopedCudaAlloc<A: CudaAlloc, B: CudaAlloc>(A, B);
impl<A: CudaAlloc, B: CudaAlloc> private::Sealed for ScopedCudaAlloc<A, B> {}
impl<A: CudaAlloc, B: CudaAlloc> ScopedCudaAlloc<A, B> {
    pub fn new(front: A, tail: B) -> Self {
        Self(front, tail)
    }
}

pub struct CudaDropWrapper<C: private::SealedCudaDrop>(Option<C>);
impl<C: private::SealedCudaDrop> From<C> for CudaDropWrapper<C> {
    fn from(val: C) -> Self {
        Self(Some(val))
    }
}
impl<C: private::SealedCudaDrop> Drop for CudaDropWrapper<C> {
    fn drop(&mut self) {
        if let Some(val) = self.0.take() {
            if let Err((_err, val)) = C::drop(val) {
                core::mem::forget(val)
            }
        }
    }
}
impl<C: private::SealedCudaDrop> Deref for CudaDropWrapper<C> {
    type Target = C;

    fn deref(&self) -> &Self::Target {
        self.0.as_ref().unwrap()
    }
}
impl<C: private::SealedCudaDrop> DerefMut for CudaDropWrapper<C> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.0.as_mut().unwrap()
    }
}
impl<C: private::SealedCudaDrop> private::Sealed for CudaDropWrapper<C> {}

impl<C: DeviceCopy> private::SealedCudaDrop for DeviceBuffer<C> {
    fn drop(val: Self) -> Result<(), (CudaError, Self)> {
        Self::drop(val)
    }
}

pub trait CudaBorrow {
    type CudaRepresentation: DeviceCopy;
    type CudaAllocation: CudaAlloc;

    /// # Errors
    /// Returns a `rustacuda::errors::CudaError` iff an error occurs inside CUDA
    fn borrow<A: CudaAlloc>(
        &self,
        alloc: A,
    ) -> CudaResult<(
        Self::CudaRepresentation,
        ScopedCudaAlloc<Self::CudaAllocation, A>,
    )>;
}
