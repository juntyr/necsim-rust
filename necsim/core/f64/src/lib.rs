#![deny(clippy::pedantic)]
#![no_std]

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

#[macro_export]
macro_rules! link {
    ($func:ident => $item:item) => {
        #[inline]
        #[export_name = concat!("necsim_core_f64_", stringify!($func))]
        $item
    };
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
        extern "Rust" {
            #[link_name = "necsim_core_f64_floor"]
            fn floor_impl(x: f64) -> f64;
        }

        unsafe { floor_impl(self) }
    }

    #[inline]
    fn ceil(self) -> f64 {
        extern "Rust" {
            #[link_name = "necsim_core_f64_ceil"]
            fn ceil_impl(x: f64) -> f64;
        }

        unsafe { ceil_impl(self) }
    }

    #[inline]
    fn ln(self) -> f64 {
        extern "Rust" {
            #[link_name = "necsim_core_f64_ln"]
            fn ln_impl(x: f64) -> f64;
        }

        unsafe { ln_impl(self) }
    }

    #[inline]
    fn exp(self) -> f64 {
        extern "Rust" {
            #[link_name = "necsim_core_f64_exp"]
            fn exp_impl(x: f64) -> f64;
        }

        unsafe { exp_impl(self) }
    }

    #[inline]
    fn sqrt(self) -> f64 {
        extern "Rust" {
            #[link_name = "necsim_core_f64_sqrt"]
            fn sqrt_impl(x: f64) -> f64;
        }

        unsafe { sqrt_impl(self) }
    }

    #[inline]
    fn sin(self) -> f64 {
        extern "Rust" {
            #[link_name = "necsim_core_f64_sin"]
            fn sin_impl(x: f64) -> f64;
        }

        unsafe { sin_impl(self) }
    }

    #[inline]
    fn cos(self) -> f64 {
        extern "Rust" {
            #[link_name = "necsim_core_f64_cos"]
            fn cos_impl(x: f64) -> f64;
        }

        unsafe { cos_impl(self) }
    }

    #[inline]
    fn round(self) -> f64 {
        extern "Rust" {
            #[link_name = "necsim_core_f64_round"]
            fn round_impl(x: f64) -> f64;
        }

        unsafe { round_impl(self) }
    }
}
