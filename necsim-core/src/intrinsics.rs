#[must_use]
pub fn floor(val: f64) -> f64 {
    unsafe { core::intrinsics::floorf64(val) }
}

#[must_use]
pub fn ln(val: f64) -> f64 {
    unsafe { core::intrinsics::logf64(val) }
}
