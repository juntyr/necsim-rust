use core::{convert::TryFrom, fmt};

use serde::{Deserialize, Deserializer, Serialize};

#[derive(Debug)]
#[allow(clippy::module_name_repetitions)]
pub struct OffByOneU32Error(u64);

impl fmt::Display for OffByOneU32Error {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{} is not in {{1, .., 2^32}}.", self.0)
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, TypeLayout)]
#[repr(transparent)]
#[serde(try_from = "u64", into = "u64")]
pub struct OffByOneU32(u32);

impl fmt::Display for OffByOneU32 {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(&self.get(), fmt)
    }
}

impl OffByOneU32 {
    /// # Errors
    ///
    /// Returns `OffByOneU32Error` if not `1 <= value <= 2^32`
    pub const fn new(value: u64) -> Result<Self, OffByOneU32Error> {
        match u32::try_from(value.wrapping_sub(1)) {
            Ok(value) => Ok(Self(value)),
            Err(_) => Err(OffByOneU32Error(value)),
        }
    }

    #[must_use]
    /// Creates a off-by-one u32 without checking the value.
    ///
    /// # Safety
    ///
    /// The value must be in {1, .., 2^32}.
    pub const unsafe fn new_unchecked(value: u64) -> Self {
        #[allow(clippy::cast_possible_truncation)]
        Self(value.wrapping_sub(1) as u32)
    }

    #[must_use]
    pub const fn get(self) -> u64 {
        u64::from(self)
    }

    #[must_use]
    pub const fn add_incl(self, other: u32) -> u32 {
        other.wrapping_add(self.0)
    }

    #[must_use]
    pub const fn add_excl(self, other: u32) -> u32 {
        other.wrapping_add(self.0).wrapping_add(1)
    }

    #[must_use]
    pub const fn one() -> Self {
        Self(0)
    }

    #[must_use]
    pub const fn max() -> Self {
        Self(u32::MAX)
    }

    #[must_use]
    pub const fn inv(self) -> u32 {
        u32::MAX - self.0
    }
}

impl TryFrom<u64> for OffByOneU32 {
    type Error = OffByOneU32Error;

    fn try_from(value: u64) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl const From<OffByOneU32> for u64 {
    fn from(val: OffByOneU32) -> Self {
        u64::from(val.0) + 1_u64
    }
}

impl const From<OffByOneU32> for i64 {
    fn from(val: OffByOneU32) -> Self {
        i64::from(val.0) + 1_i64
    }
}

impl const From<OffByOneU32> for f64 {
    fn from(val: OffByOneU32) -> Self {
        f64::from(val.0) + 1.0_f64
    }
}

impl const From<OffByOneU32> for usize {
    fn from(val: OffByOneU32) -> Self {
        (val.0 as usize) + 1_usize
    }
}

impl<'de> Deserialize<'de> for OffByOneU32 {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Self::new(u64::deserialize(deserializer)?).map_err(serde::de::Error::custom)
    }
}
