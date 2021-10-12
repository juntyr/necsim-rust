necsim_core_f64::link! { floor => unsafe fn intrinsics_floor(x: f64) -> f64 {
    core::intrinsics::floorf64(x)
} }

necsim_core_f64::link! { ceil => unsafe fn intrinsics_ceil(x: f64) -> f64 {
    core::intrinsics::ceilf64(x)
} }

necsim_core_f64::link! { ln => unsafe fn intrinsics_ln(x: f64) -> f64 {
    core::intrinsics::logf64(x)
} }

necsim_core_f64::link! { exp => unsafe fn intrinsics_exp(x: f64) -> f64 {
    core::intrinsics::expf64(x)
} }

necsim_core_f64::link! { sqrt => unsafe fn intrinsics_sqrt(x: f64) -> f64 {
    core::intrinsics::sqrtf64(x)
} }

necsim_core_f64::link! { sin => unsafe fn intrinsics_sin(x: f64) -> f64 {
    core::intrinsics::sinf64(x)
} }

necsim_core_f64::link! { cos => unsafe fn intrinsics_cos(x: f64) -> f64 {
    core::intrinsics::cosf64(x)
} }

necsim_core_f64::link! { round => unsafe fn intrinsics_round(x: f64) -> f64 {
    core::intrinsics::roundf64(x)
} }
