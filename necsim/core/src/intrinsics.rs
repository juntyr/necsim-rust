#[must_use]
#[inline]
pub fn floor(val: f64) -> f64 {
    unsafe { core::intrinsics::floorf64(val) }
}

#[must_use]
#[inline]
pub fn ceil(val: f64) -> f64 {
    unsafe { core::intrinsics::ceilf64(val) }
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

#[must_use]
#[inline]
pub fn exp(val: f64) -> f64 {
    #[cfg(not(target_os = "cuda"))]
    unsafe {
        core::intrinsics::expf64(val)
    }
    #[cfg(target_os = "cuda")]
    unsafe {
        rust_cuda::device::nvptx::_exp(val)
    }
}

#[must_use]
#[inline]
pub fn sqrt(val: f64) -> f64 {
    unsafe { core::intrinsics::sqrtf64(val) }
}

#[must_use]
#[inline]
pub fn sin(val: f64) -> f64 {
    #[cfg(not(target_os = "cuda"))]
    unsafe {
        core::intrinsics::sinf64(val)
    }
    #[cfg(target_os = "cuda")]
    unsafe {
        rust_cuda::device::nvptx::_sin(val)
    }
}

#[must_use]
#[inline]
pub fn cos(val: f64) -> f64 {
    #[cfg(not(target_os = "cuda"))]
    unsafe {
        core::intrinsics::cosf64(val)
    }
    #[cfg(target_os = "cuda")]
    unsafe {
        rust_cuda::device::nvptx::_cos(val)
    }
}

#[must_use]
#[inline]
pub fn round(val: f64) -> f64 {
    #[cfg(not(target_os = "cuda"))]
    unsafe {
        core::intrinsics::roundf64(val)
    }
    #[cfg(target_os = "cuda")]
    unsafe {
        rust_cuda::device::nvptx::_round(val)
    }
}
