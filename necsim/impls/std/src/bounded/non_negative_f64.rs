use std::convert::TryFrom;

use serde::Deserialize;
use thiserror::Error;

#[derive(Debug, Error)]
#[error("{0} is negative.")]
#[allow(clippy::module_name_repetitions)]
pub struct NonNegativeF64Error(f64);

#[derive(Copy, Clone, Deserialize)]
#[serde(try_from = "f64")]
pub struct NonNegativeF64(f64);

impl TryFrom<f64> for NonNegativeF64 {
    type Error = NonNegativeF64Error;

    fn try_from(value: f64) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl std::fmt::Debug for NonNegativeF64 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        struct NonNegativeF64Range(f64);

        impl std::fmt::Debug for NonNegativeF64Range {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "0.0 <= {}", self.0)
            }
        }

        f.debug_tuple("NonNegativeF64")
            .field(&NonNegativeF64Range(self.0))
            .finish()
    }
}

impl NonNegativeF64 {
    /// # Errors
    ///
    /// Returns `NonNegativeF64Error` if not `0.0 <= value`
    pub fn new(value: f64) -> Result<Self, NonNegativeF64Error> {
        if value >= 0.0 {
            Ok(Self(value))
        } else {
            Err(NonNegativeF64Error(value))
        }
    }

    #[must_use]
    pub fn get(self) -> f64 {
        self.0
    }
}
