use necsim_core::cogs::MathsCore;

#[derive(Clone, Debug)]
#[allow(clippy::module_name_repetitions)]
pub enum NvptxMathsCore {}

// TODO: Ensure consistency for cached calculations on the CPU,
//        maybe guard with linker error?
impl MathsCore for NvptxMathsCore {
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
        #[cfg(not(target_os = "cuda"))]
        unsafe {
            core::intrinsics::logf64(x)
        }
        #[cfg(target_os = "cuda")]
        unsafe {
            const FRAC_1_LOG2_E: f64 = 1.0_f64 / core::f64::consts::LOG2_E;

            #[allow(clippy::cast_possible_truncation)]
            let x: f32 = x as f32;
            let f: f32;

            core::arch::asm!("lg2.approx.f32 {}, {};", out(reg32) f, in(reg32) x, options(pure, nomem, nostack));

            // f / log_2(e)
            f64::from(f) * FRAC_1_LOG2_E
        }
    }

    #[inline]
    fn exp(x: f64) -> f64 {
        #[cfg(not(target_os = "cuda"))]
        unsafe {
            core::intrinsics::expf64(x)
        }
        #[cfg(target_os = "cuda")]
        unsafe {
            #[allow(clippy::cast_possible_truncation)]
            let x: f32 = (x * core::f64::consts::LOG2_E) as f32;
            let f: f32;

            core::arch::asm!("ex2.approx.f32 {}, {};", out(reg32) f, in(reg32) x, options(pure, nomem, nostack));

            f64::from(f)
        }
    }

    #[inline]
    fn sqrt(x: f64) -> f64 {
        unsafe { core::intrinsics::sqrtf64(x) }
    }

    #[inline]
    fn sin(x: f64) -> f64 {
        #[cfg(not(target_os = "cuda"))]
        unsafe {
            core::intrinsics::sinf64(x)
        }
        #[cfg(target_os = "cuda")]
        unsafe {
            #[allow(clippy::cast_possible_truncation)]
            let x: f32 = x as f32;
            let f: f32;

            core::arch::asm!("sin.approx.f32 {}, {};", out(reg32) f, in(reg32) x, options(pure, nomem, nostack));

            f64::from(f)
        }
    }

    #[inline]
    fn cos(x: f64) -> f64 {
        #[cfg(not(target_os = "cuda"))]
        unsafe {
            core::intrinsics::cosf64(x)
        }
        #[cfg(target_os = "cuda")]
        unsafe {
            #[allow(clippy::cast_possible_truncation)]
            let x: f32 = x as f32;
            let f: f32;

            core::arch::asm!("cos.approx.f32 {}, {};", out(reg32) f, in(reg32) x, options(pure, nomem, nostack));

            f64::from(f)
        }
    }

    #[inline]
    fn round(x: f64) -> f64 {
        #[cfg(not(target_os = "cuda"))]
        unsafe {
            core::intrinsics::roundf64(x)
        }
        #[cfg(target_os = "cuda")]
        unsafe {
            const ROUND_TRUNC_OFFSET: f64 = 0.5_f64 - 0.25_f64 * f64::EPSILON;

            let offset: f64;
            core::arch::asm!("copysign {}, {}, {};", out(reg64) offset, in(reg64) x, const ROUND_TRUNC_OFFSET.to_bits(), options(pure, nomem, nostack));

            let overshot = x + offset;

            let round: f64;
            core::arch::asm!("cvt.rzi.f64.f64 {}, {};", out(reg64) round, in(reg64) overshot, options(pure, nomem, nostack));

            round
        }
    }
}
