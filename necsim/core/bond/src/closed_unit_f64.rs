use core::{
    cmp::Ordering,
    convert::TryFrom,
    fmt,
    hash::{Hash, Hasher},
    ops::Mul,
};

use serde::{Deserialize, Serialize};

use crate::PositiveUnitF64;

#[derive(Debug)]
#[allow(clippy::module_name_repetitions)]
pub struct ClosedUnitF64Error(f64);

impl fmt::Display for ClosedUnitF64Error {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{} is not in [0.0, 1.0].", self.0)
    }
}

#[allow(clippy::unsafe_derive_deserialize)]
#[derive(Copy, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "cuda", derive(rust_cuda::rustacuda_core::DeviceCopy))]
#[cfg_attr(feature = "cuda", rustacuda(core = "rust_cuda::rustacuda_core"))]
#[cfg_attr(feature = "mpi", derive(mpi::traits::Equivalence))]
#[repr(transparent)]
#[serde(try_from = "f64")]
pub struct ClosedUnitF64(f64);

impl TryFrom<f64> for ClosedUnitF64 {
    type Error = ClosedUnitF64Error;

    fn try_from(value: f64) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl fmt::Debug for ClosedUnitF64 {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        struct ClosedUnitF64Range(f64);

        impl fmt::Debug for ClosedUnitF64Range {
            fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
                write!(fmt, "0.0 <= {} <= 1.0", self.0)
            }
        }

        fmt.debug_tuple("ClosedUnitF64")
            .field(&ClosedUnitF64Range(self.0))
            .finish()
    }
}

impl ClosedUnitF64 {
    /// # Errors
    ///
    /// Returns `ClosedUnitF64Error` if not `0.0 <= value <= 1.0`
    pub fn new(value: f64) -> Result<Self, ClosedUnitF64Error> {
        if (0.0..=1.0).contains(&value) {
            Ok(Self(value))
        } else {
            Err(ClosedUnitF64Error(value))
        }
    }

    /// # Safety
    ///
    /// Only safe iff `0.0 <= value <= 1.0`
    #[must_use]
    pub unsafe fn new_unchecked(value: f64) -> Self {
        Self(value)
    }

    #[must_use]
    pub fn zero() -> Self {
        Self(0.0_f64)
    }

    #[must_use]
    pub fn one() -> Self {
        Self(1.0_f64)
    }

    #[must_use]
    pub fn get(self) -> f64 {
        self.0
    }

    #[must_use]
    pub fn one_minus(self) -> Self {
        Self(1.0_f64 - self.0)
    }
}

impl From<PositiveUnitF64> for ClosedUnitF64 {
    fn from(value: PositiveUnitF64) -> Self {
        Self(value.get())
    }
}

impl PartialEq for ClosedUnitF64 {
    fn eq(&self, other: &Self) -> bool {
        self.0.eq(&other.0)
    }
}

impl Eq for ClosedUnitF64 {}

impl PartialOrd for ClosedUnitF64 {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.0.partial_cmp(&other.0)
    }
}

impl Ord for ClosedUnitF64 {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.total_cmp(&other.0)
    }
}

impl Hash for ClosedUnitF64 {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.to_bits().hash(state);
    }
}

impl PartialEq<f64> for ClosedUnitF64 {
    fn eq(&self, other: &f64) -> bool {
        self.0.eq(other)
    }
}

impl PartialOrd<f64> for ClosedUnitF64 {
    fn partial_cmp(&self, other: &f64) -> Option<Ordering> {
        self.0.partial_cmp(other)
    }
}

impl Mul for ClosedUnitF64 {
    type Output = Self;

    fn mul(self, other: Self) -> Self {
        Self(self.0 * other.0)
    }
}
