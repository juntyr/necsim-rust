use core::{
    cmp::Ordering,
    convert::TryFrom,
    fmt,
    hash::{Hash, Hasher},
};

use serde::{Deserialize, Serialize};

#[derive(Debug)]
#[allow(clippy::module_name_repetitions)]
pub struct ZeroExclOneInclF64Error(f64);

impl fmt::Display for ZeroExclOneInclF64Error {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{} is not in (0.0, 1.0].", self.0)
    }
}

#[derive(Copy, Clone, Deserialize, Serialize)]
#[cfg_attr(feature = "cuda", derive(rustacuda_derive::DeviceCopy))]
#[cfg_attr(feature = "mpi", derive(mpi::traits::Equivalence))]
#[repr(transparent)]
#[serde(try_from = "f64")]
pub struct ZeroExclOneInclF64(f64);

impl TryFrom<f64> for ZeroExclOneInclF64 {
    type Error = ZeroExclOneInclF64Error;

    fn try_from(value: f64) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl fmt::Debug for ZeroExclOneInclF64 {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        struct ZeroExclOneInclF64Range(f64);

        impl fmt::Debug for ZeroExclOneInclF64Range {
            fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
                write!(fmt, "0.0 < {} <= 1.0", self.0)
            }
        }

        fmt.debug_tuple("ZeroExclOneInclF64")
            .field(&ZeroExclOneInclF64Range(self.0))
            .finish()
    }
}

impl ZeroExclOneInclF64 {
    /// # Errors
    ///
    /// Returns `ZeroExclOneInclF64Error` if not `0.0 < value <= 1.0`
    pub fn new(value: f64) -> Result<Self, ZeroExclOneInclF64Error> {
        if value > 0.0 && value <= 1.0 {
            Ok(Self(value))
        } else {
            Err(ZeroExclOneInclF64Error(value))
        }
    }

    #[must_use]
    pub fn get(self) -> f64 {
        self.0
    }
}

impl PartialEq for ZeroExclOneInclF64 {
    fn eq(&self, other: &Self) -> bool {
        self.0.eq(&other.0)
    }
}

impl Eq for ZeroExclOneInclF64 {}

impl PartialOrd for ZeroExclOneInclF64 {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.0.partial_cmp(&other.0)
    }
}

impl Ord for ZeroExclOneInclF64 {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.total_cmp(&other.0)
    }
}

impl Hash for ZeroExclOneInclF64 {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.to_bits().hash(state)
    }
}

impl PartialEq<f64> for ZeroExclOneInclF64 {
    fn eq(&self, other: &f64) -> bool {
        self.0.eq(&other)
    }
}

impl PartialOrd<f64> for ZeroExclOneInclF64 {
    fn partial_cmp(&self, other: &f64) -> Option<Ordering> {
        self.0.partial_cmp(&other)
    }
}
