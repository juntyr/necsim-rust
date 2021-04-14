use core::{convert::TryFrom, fmt};

use serde::{Deserialize, Deserializer, Serialize};

#[derive(Debug)]
#[allow(clippy::module_name_repetitions)]
pub struct NonZeroOneU64Error(u64);

impl fmt::Display for NonZeroOneU64Error {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{} is zero or one.", self.0)
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize)]
#[repr(transparent)]
#[rustc_layout_scalar_valid_range_start(2)]
#[rustc_nonnull_optimization_guaranteed]
#[serde(try_from = "u64")]
pub struct NonZeroOneU64(u64);

impl NonZeroOneU64 {
    /// # Errors
    ///
    /// Returns `NonZeroOneU64Error` if not `1 < value`
    pub fn new(value: u64) -> Result<Self, NonZeroOneU64Error> {
        if value > 1 {
            Ok(unsafe { Self(value) })
        } else {
            Err(NonZeroOneU64Error(value))
        }
    }

    #[must_use]
    /// Creates a non-zero, non-one u64 without checking the value.
    ///
    /// # Safety
    ///
    /// The value must not be zero or one.
    pub unsafe fn new_unchecked(value: u64) -> Self {
        Self(value)
    }

    #[must_use]
    pub fn get(self) -> u64 {
        self.0
    }
}

impl TryFrom<u64> for NonZeroOneU64 {
    type Error = NonZeroOneU64Error;

    fn try_from(value: u64) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl<'de> Deserialize<'de> for NonZeroOneU64 {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Self::new(u64::deserialize(deserializer)?).map_err(serde::de::Error::custom)
    }
}
