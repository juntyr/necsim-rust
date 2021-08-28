use core::{
    cmp::Ordering,
    convert::TryFrom,
    fmt,
    hash::{Hash, Hasher},
};

use serde::{Deserialize, Serialize};

#[derive(Debug)]
#[allow(clippy::module_name_repetitions)]
pub struct PositiveUnitF64Error(f64);

impl fmt::Display for PositiveUnitF64Error {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{} is not in (0.0, 1.0].", self.0)
    }
}

#[derive(Copy, Clone, Deserialize, Serialize)]
#[repr(transparent)]
#[serde(try_from = "f64")]
pub struct PositiveUnitF64(f64);

impl TryFrom<f64> for PositiveUnitF64 {
    type Error = PositiveUnitF64Error;

    fn try_from(value: f64) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl fmt::Debug for PositiveUnitF64 {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        struct PositiveUnitF64Range(f64);

        impl fmt::Debug for PositiveUnitF64Range {
            fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
                write!(fmt, "0.0 < {} <= 1.0", self.0)
            }
        }

        fmt.debug_tuple("PositiveUnitF64")
            .field(&PositiveUnitF64Range(self.0))
            .finish()
    }
}

impl PositiveUnitF64 {
    /// # Errors
    ///
    /// Returns `PositiveUnitF64Error` if not `0.0 < value <= 1.0`
    pub const fn new(value: f64) -> Result<Self, PositiveUnitF64Error> {
        if value > 0.0 && value <= 1.0 {
            Ok(Self(value))
        } else {
            Err(PositiveUnitF64Error(value))
        }
    }

    #[must_use]
    pub const fn get(self) -> f64 {
        self.0
    }
}

impl PartialEq for PositiveUnitF64 {
    fn eq(&self, other: &Self) -> bool {
        self.0.eq(&other.0)
    }
}

impl Eq for PositiveUnitF64 {}

impl PartialOrd for PositiveUnitF64 {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.0.partial_cmp(&other.0)
    }
}

impl Ord for PositiveUnitF64 {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.total_cmp(&other.0)
    }
}

impl Hash for PositiveUnitF64 {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.to_bits().hash(state);
    }
}

impl PartialEq<f64> for PositiveUnitF64 {
    fn eq(&self, other: &f64) -> bool {
        self.0.eq(other)
    }
}

impl PartialOrd<f64> for PositiveUnitF64 {
    fn partial_cmp(&self, other: &f64) -> Option<Ordering> {
        self.0.partial_cmp(other)
    }
}
