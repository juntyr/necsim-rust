#![deny(clippy::pedantic)]
#![no_std]
#![feature(core_intrinsics)]

#[macro_use]
extern crate default_env;

pub trait F64Core {
    #[must_use]
    fn floor(self) -> f64;
    #[must_use]
    fn ceil(self) -> f64;
    #[must_use]
    fn ln(self) -> f64;
    #[must_use]
    fn exp(self) -> f64;
    #[must_use]
    fn sqrt(self) -> f64;
    #[must_use]
    fn sin(self) -> f64;
    #[must_use]
    fn cos(self) -> f64;
    #[must_use]
    fn round(self) -> f64;
}

#[must_use]
#[inline]
pub fn floor(val: f64) -> f64 {
    F64Core::floor(val)
}

#[must_use]
#[inline]
pub fn ceil(val: f64) -> f64 {
    F64Core::ceil(val)
}

#[must_use]
#[inline]
pub fn ln(val: f64) -> f64 {
    F64Core::ln(val)
}

#[must_use]
#[inline]
pub fn exp(val: f64) -> f64 {
    F64Core::exp(val)
}

#[must_use]
#[inline]
pub fn sqrt(val: f64) -> f64 {
    F64Core::sqrt(val)
}

#[must_use]
#[inline]
pub fn sin(val: f64) -> f64 {
    F64Core::sin(val)
}

#[must_use]
#[inline]
pub fn cos(val: f64) -> f64 {
    F64Core::cos(val)
}

#[must_use]
#[inline]
pub fn round(val: f64) -> f64 {
    F64Core::round(val)
}

impl F64Core for f64 {
    #[inline]
    fn floor(self) -> f64 {
        #[no_mangle]
        #[inline]
        unsafe fn _floor_intrinsic(x: f64) -> f64 {
            core::intrinsics::floorf64(x)
        }

        extern "Rust" {
            #[link_name = default_env!("NECSIM_CORE_F64_LINK_FLOOR", "_floor_intrinsic")]
            fn floor_impl(x: f64) -> f64;
        }

        unsafe { floor_impl(self) }
    }

    #[inline]
    fn ceil(self) -> f64 {
        #[no_mangle]
        #[inline]
        unsafe fn _ceil_intrinsic(x: f64) -> f64 {
            core::intrinsics::ceilf64(x)
        }

        extern "Rust" {
            #[link_name = default_env!("NECSIM_CORE_F64_LINK_CEIL", "_ceil_intrinsic")]
            fn ceil_impl(x: f64) -> f64;
        }

        unsafe { ceil_impl(self) }
    }

    #[inline]
    fn ln(self) -> f64 {
        #[no_mangle]
        #[inline]
        unsafe fn _ln_intrinsic(x: f64) -> f64 {
            core::intrinsics::logf64(x)
        }

        extern "Rust" {
            #[link_name = default_env!("NECSIM_CORE_F64_LINK_LN", "_ln_intrinsic")]
            fn ln_impl(x: f64) -> f64;
        }

        unsafe { ln_impl(self) }
    }

    #[inline]
    fn exp(self) -> f64 {
        #[no_mangle]
        #[inline]
        unsafe fn _exp_intrinsic(x: f64) -> f64 {
            core::intrinsics::expf64(x)
        }

        extern "Rust" {
            #[link_name = default_env!("NECSIM_CORE_F64_LINK_EXP", "_exp_intrinsic")]
            fn exp_impl(x: f64) -> f64;
        }

        unsafe { exp_impl(self) }
    }

    #[inline]
    fn sqrt(self) -> f64 {
        #[no_mangle]
        #[inline]
        unsafe fn _sqrt_intrinsic(x: f64) -> f64 {
            core::intrinsics::sqrtf64(x)
        }

        extern "Rust" {
            #[link_name = default_env!("NECSIM_CORE_F64_LINK_SQRT", "_sqrt_intrinsic")]
            fn sqrt_impl(x: f64) -> f64;
        }

        unsafe { sqrt_impl(self) }
    }

    #[inline]
    fn sin(self) -> f64 {
        #[no_mangle]
        #[inline]
        unsafe fn _sin_intrinsic(x: f64) -> f64 {
            core::intrinsics::sinf64(x)
        }

        extern "Rust" {
            #[link_name = default_env!("NECSIM_CORE_F64_LINK_SIN", "_sin_intrinsic")]
            fn sin_impl(x: f64) -> f64;
        }

        unsafe { sin_impl(self) }
    }

    #[inline]
    fn cos(self) -> f64 {
        #[no_mangle]
        #[inline]
        unsafe fn _cos_intrinsic(x: f64) -> f64 {
            core::intrinsics::cosf64(x)
        }

        extern "Rust" {
            #[link_name = default_env!("NECSIM_CORE_F64_LINK_COS", "_cos_intrinsic")]
            fn cos_impl(x: f64) -> f64;
        }

        unsafe { cos_impl(self) }
    }

    #[inline]
    fn round(self) -> f64 {
        #[no_mangle]
        #[inline]
        unsafe fn _round_intrinsic(x: f64) -> f64 {
            core::intrinsics::roundf64(x)
        }

        extern "Rust" {
            #[link_name = default_env!("NECSIM_CORE_F64_LINK_ROUND", "_round_intrinsic")]
            fn round_impl(x: f64) -> f64;
        }

        unsafe { round_impl(self) }
    }
}
