#![deny(clippy::pedantic)]
#![no_std]

#[cfg(feature = "cpu")]
use core::ops::{Deref, DerefMut};

#[cfg(feature = "cpu")]
use rustacuda::error::{CudaError, CudaResult};
#[cfg(feature = "cpu")]
use rustacuda::memory::{DeviceBox, DeviceBuffer};
#[cfg(feature = "cpu")]
use rustacuda_core::DevicePointer;

use rustacuda_core::DeviceCopy;

#[cfg(feature = "cpu")]
pub(crate) mod private {
    pub trait Sealed {}

    pub trait SealedCudaDrop: Sized {
        fn drop(val: Self) -> Result<(), (super::CudaError, Self)>;
    }
}

#[cfg(feature = "cpu")]
pub trait CudaAlloc: private::Sealed {}
#[cfg(feature = "cpu")]
impl<T: private::Sealed> CudaAlloc for T {}

#[cfg(feature = "cpu")]
pub struct NullCudaAlloc;
#[cfg(feature = "cpu")]
impl private::Sealed for NullCudaAlloc {}

#[cfg(feature = "cpu")]
pub struct ScopedCudaAlloc<A: CudaAlloc, B: CudaAlloc>(A, B);
#[cfg(feature = "cpu")]
impl<A: CudaAlloc, B: CudaAlloc> private::Sealed for ScopedCudaAlloc<A, B> {}
#[cfg(feature = "cpu")]
impl<A: CudaAlloc, B: CudaAlloc> ScopedCudaAlloc<A, B> {
    pub fn new(front: A, tail: B) -> Self {
        Self(front, tail)
    }
}

#[cfg(feature = "cpu")]
pub struct CudaDropWrapper<C: private::SealedCudaDrop>(Option<C>);
#[cfg(feature = "cpu")]
impl<C: private::SealedCudaDrop> From<C> for CudaDropWrapper<C> {
    fn from(val: C) -> Self {
        Self(Some(val))
    }
}
#[cfg(feature = "cpu")]
impl<C: private::SealedCudaDrop> Drop for CudaDropWrapper<C> {
    fn drop(&mut self) {
        if let Some(val) = self.0.take() {
            if let Err((_err, val)) = C::drop(val) {
                core::mem::forget(val)
            }
        }
    }
}
#[cfg(feature = "cpu")]
impl<C: private::SealedCudaDrop> Deref for CudaDropWrapper<C> {
    type Target = C;

    fn deref(&self) -> &Self::Target {
        self.0.as_ref().unwrap()
    }
}
#[cfg(feature = "cpu")]
impl<C: private::SealedCudaDrop> DerefMut for CudaDropWrapper<C> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.0.as_mut().unwrap()
    }
}
#[cfg(feature = "cpu")]
impl<C: private::SealedCudaDrop> private::Sealed for CudaDropWrapper<C> {}

#[cfg(feature = "cpu")]
impl<C: DeviceCopy> private::SealedCudaDrop for DeviceBuffer<C> {
    fn drop(val: Self) -> Result<(), (CudaError, Self)> {
        Self::drop(val)
    }
}

#[cfg(feature = "cpu")]
impl<C: DeviceCopy> private::SealedCudaDrop for DeviceBox<C> {
    fn drop(val: Self) -> Result<(), (CudaError, Self)> {
        Self::drop(val)
    }
}

/// # Safety
/// This is an internal trait and should ONLY be derived automatically using `#[derive(RustToCuda)]`
pub unsafe trait RustToCuda {
    type CudaRepresentation: DeviceCopy + CudaAsRust<RustRepresentation = Self>;

    #[cfg(feature = "cpu")]
    type CudaAllocation: CudaAlloc;

    #[cfg(feature = "cpu")]
    /// # Errors
    /// Returns a `rustacuda::errors::CudaError` iff an error occurs inside CUDA
    /// # Safety
    /// This is an internal function and should NEVER be called manually
    #[allow(clippy::type_complexity)]
    unsafe fn borrow<A: CudaAlloc>(
        &self,
        alloc: A,
    ) -> CudaResult<(
        Self::CudaRepresentation,
        ScopedCudaAlloc<Self::CudaAllocation, A>,
    )>;
}

/// # Safety
/// This is an internal trait and should NEVER be implemented manually
pub unsafe trait CudaAsRust {
    type RustRepresentation: RustToCuda<CudaRepresentation = Self>;

    #[cfg(not(feature = "cpu"))]
    /// # Safety
    /// This is an internal function and should NEVER be called manually
    unsafe fn as_rust(&self) -> Self::RustRepresentation;
}

#[cfg(feature = "cpu")]
/// # Safety
/// This trait should ONLY be derived automatically using `#[derive(LendToCuda)]`
pub unsafe trait LendToCuda: RustToCuda {
    /// # Errors
    /// Returns a `rustacuda::errors::CudaError` iff an error occurs inside CUDA
    fn lend_to_cuda<
        O,
        F: FnOnce(DevicePointer<<Self as RustToCuda>::CudaRepresentation>) -> CudaResult<O>,
    >(
        &self,
        inner: F,
    ) -> CudaResult<O>;
}

#[cfg(not(feature = "cpu"))]
/// # Safety
/// This is an internal trait and should NEVER be implemented manually
pub unsafe trait BorrowFromRust: RustToCuda {
    /// # Safety
    /// This function is only safe to call iff `this` is the `DevicePointer` borrowed on the CPU
    /// using the corresponding `LendToCuda::lend_to_cuda`.
    unsafe fn with_borrow_from_rust<O, F: FnOnce(&Self) -> O>(
        cuda_repr_ptr: *const <Self as RustToCuda>::CudaRepresentation,
        inner: F,
    ) -> O;
}
