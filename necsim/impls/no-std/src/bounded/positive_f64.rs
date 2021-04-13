use core::{convert::TryFrom, fmt};

use serde::Deserialize;

#[derive(Debug)]
#[allow(clippy::module_name_repetitions)]
pub struct PositiveF64Error(f64);

impl fmt::Display for PositiveF64Error {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{} is not positive.", self.0)
    }
}

#[derive(Copy, Clone, Deserialize)]
#[serde(try_from = "f64")]
pub struct PositiveF64(f64);

impl TryFrom<f64> for PositiveF64 {
    type Error = PositiveF64Error;

    fn try_from(value: f64) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl fmt::Debug for PositiveF64 {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        struct PositiveF64Range(f64);

        impl fmt::Debug for PositiveF64Range {
            fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
                write!(fmt, "0.0 < {}", self.0)
            }
        }

        fmt.debug_tuple("PositiveF64")
            .field(&PositiveF64Range(self.0))
            .finish()
    }
}

impl PositiveF64 {
    /// # Errors
    ///
    /// Returns `PositiveF64Error` if not `0.0 < value`
    pub fn new(value: f64) -> Result<Self, PositiveF64Error> {
        if value > 0.0 {
            Ok(Self(value))
        } else {
            Err(PositiveF64Error(value))
        }
    }

    #[must_use]
    pub fn get(self) -> f64 {
        self.0
    }
}
