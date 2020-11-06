#[must_use]
#[inline]
pub fn floor(val: f64) -> f64 {
    unsafe { core::intrinsics::floorf64(val) }
}

#[must_use]
#[inline]
pub fn ln(val: f64) -> f64 {
    #[cfg(not(target_os = "cuda"))]
    unsafe {
        core::intrinsics::logf64(val)
    }
    #[cfg(target_os = "cuda")]
    unsafe {
        rust_cuda::device::nvptx::_log(val)
    }
}
