use core::mem::MaybeUninit;

use rust_cuda::{rustacuda_core::DeviceCopy, utils::stack::StackOnly};

#[repr(C)]
#[doc(hidden)]
pub struct MaybeSome<T: StackOnly>(MaybeUninit<T>);

// Safety: Any type that is fully on the stack without any references
//         to the heap can be safely copied to the GPU
unsafe impl<T: StackOnly> DeviceCopy for MaybeSome<T> {}

impl<T: StackOnly> MaybeSome<T> {
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