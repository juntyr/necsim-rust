use std::convert::TryFrom;

use serde::Deserialize;
use thiserror::Error;

#[derive(Debug, Error)]
#[error("{0} is not in (0.0, 1.0].")]
#[allow(clippy::module_name_repetitions)]
pub struct ZeroExclOneInclF64Error(f64);

#[derive(Copy, Clone, Deserialize)]
#[serde(try_from = "f64")]
pub struct ZeroExclOneInclF64(f64);

impl TryFrom<f64> for ZeroExclOneInclF64 {
    type Error = ZeroExclOneInclF64Error;

    fn try_from(value: f64) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl std::fmt::Debug for ZeroExclOneInclF64 {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        struct ZeroExclOneInclF64Range(f64);

        impl std::fmt::Debug for ZeroExclOneInclF64Range {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(f, "0.0 < {} <= 1.0", self.0)
            }
        }

        f.debug_tuple("ZeroExclOneInclF64")
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
