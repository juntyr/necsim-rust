use std::convert::TryFrom;

use serde::Deserialize;
use thiserror::Error;

#[derive(Debug, Error)]
#[error("{0} is not in [0.0, 1.0].")]
#[allow(clippy::module_name_repetitions)]
pub struct ZeroInclOneInclF64Error(f64);

#[derive(Copy, Clone, Deserialize)]
#[serde(try_from = "f64")]
pub struct ZeroInclOneInclF64(f64);

impl TryFrom<f64> for ZeroInclOneInclF64 {
    type Error = ZeroInclOneInclF64Error;

    fn try_from(value: f64) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl std::fmt::Debug for ZeroInclOneInclF64 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        struct ZeroInclOneInclF64Range(f64);

        impl std::fmt::Debug for ZeroInclOneInclF64Range {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "0.0 <= {} <= 1.0", self.0)
            }
        }

        f.debug_tuple("ZeroInclOneInclF64")
            .field(&ZeroInclOneInclF64Range(self.0))
            .finish()
    }
}

impl ZeroInclOneInclF64 {
    /// # Errors
    ///
    /// Returns `ZeroInclOneInclF64Error` if not `0.0 <= value <= 1.0`
    pub fn new(value: f64) -> Result<Self, ZeroInclOneInclF64Error> {
        if (0.0..=1.0).contains(&value) {
            Ok(Self(value))
        } else {
            Err(ZeroInclOneInclF64Error(value))
        }
    }

    #[must_use]
    pub fn get(self) -> f64 {
        self.0
    }
}
