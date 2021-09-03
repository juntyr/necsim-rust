use necsim_core_bond::{ClosedUnitF64, NonNegativeF64};

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
    unsafe { core::intrinsics::logf64(val) }
}

#[must_use]
#[inline]
pub fn log2(val: f64) -> f64 {
    unsafe { core::intrinsics::log2f64(val) }
}

#[must_use]
#[inline]
pub fn exp(val: f64) -> f64 {
    unsafe { core::intrinsics::expf64(val) }
}

#[must_use]
#[inline]
pub fn neg_exp(val: NonNegativeF64) -> ClosedUnitF64 {
    unsafe { ClosedUnitF64::new_unchecked(-val.get()) }
}

#[must_use]
#[inline]
pub fn sqrt(val: f64) -> f64 {
    unsafe { core::intrinsics::sqrtf64(val) }
}

#[must_use]
#[inline]
pub fn safe_sqrt(val: NonNegativeF64) -> NonNegativeF64 {
    unsafe { NonNegativeF64::new_unchecked(core::intrinsics::sqrtf64(val.get())) }
}

#[must_use]
#[inline]
pub fn sin(val: f64) -> f64 {
    unsafe { core::intrinsics::sinf64(val) }
}

#[must_use]
#[inline]
pub fn cos(val: f64) -> f64 {
    unsafe { core::intrinsics::cosf64(val) }
}

#[must_use]
#[inline]
pub fn round(val: f64) -> f64 {
    unsafe { core::intrinsics::roundf64(val) }
}

#[must_use]
#[inline]
pub fn fract(val: f64) -> f64 {
    unsafe { val - core::intrinsics::truncf64(val) }
}
