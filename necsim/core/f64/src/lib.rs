#![deny(clippy::pedantic)]
#![no_std]
#![feature(core_intrinsics)]

pub trait F64Core: 'static + Clone + core::fmt::Debug {
    #[must_use]
    fn floor(x: f64) -> f64;
    #[must_use]
    fn ceil(x: f64) -> f64;
    #[must_use]
    fn ln(x: f64) -> f64;
    #[must_use]
    fn exp(x: f64) -> f64;
    #[must_use]
    fn sqrt(x: f64) -> f64;
    #[must_use]
    fn sin(x: f64) -> f64;
    #[must_use]
    fn cos(x: f64) -> f64;
    #[must_use]
    fn round(x: f64) -> f64;
}

#[derive(Clone, Debug)]
pub enum IntrinsicsF64Core {}

impl F64Core for IntrinsicsF64Core {
    #[inline]
    fn floor(x: f64) -> f64 {
        unsafe { core::intrinsics::floorf64(x) }
    }

    #[inline]
    fn ceil(x: f64) -> f64 {
        unsafe { core::intrinsics::ceilf64(x) }
    }

    #[inline]
    fn ln(x: f64) -> f64 {
        unsafe { core::intrinsics::logf64(x) }
    }

    #[inline]
    fn exp(x: f64) -> f64 {
        unsafe { core::intrinsics::expf64(x) }
    }

    #[inline]
    fn sqrt(x: f64) -> f64 {
        unsafe { core::intrinsics::sqrtf64(x) }
    }

    #[inline]
    fn sin(x: f64) -> f64 {
        unsafe { core::intrinsics::sinf64(x) }
    }

    #[inline]
    fn cos(x: f64) -> f64 {
        unsafe { core::intrinsics::cosf64(x) }
    }

    #[inline]
    fn round(x: f64) -> f64 {
        unsafe { core::intrinsics::roundf64(x) }
    }
}
