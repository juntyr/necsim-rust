use core::mem::MaybeUninit;

use rust_cuda::rustacuda_core::DeviceCopy;

#[repr(C)]
#[doc(hidden)]
pub struct MaybeSome<T: DeviceCopy>(MaybeUninit<T>);

unsafe impl<T: DeviceCopy> DeviceCopy for MaybeSome<T> {}

impl<T: DeviceCopy> MaybeSome<T> {
    #[cfg(not(target_os = "cuda"))]
    #[allow(non_upper_case_globals)]
    pub(crate) const None: Self = Self(MaybeUninit::uninit());

    #[allow(non_snake_case)]
    pub(crate) fn Some(value: T) -> Self {
        Self(MaybeUninit::new(value))
    }

    pub(crate) unsafe fn assume_some_read(&self) -> T {
        self.0.assume_init_read()
    }

    pub(crate) unsafe fn assume_some_ref(&self) -> &T {
        self.0.assume_init_ref()
    }

    pub(crate) unsafe fn assume_some_mut(&mut self) -> &mut T {
        self.0.assume_init_mut()
    }
}
