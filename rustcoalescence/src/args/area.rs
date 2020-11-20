use std::{fmt, num::ParseIntError};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseAreaError {
    TooManyDimensions,
    ParseIntError(ParseIntError),
}

impl fmt::Display for ParseAreaError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TooManyDimensions => "area can at most contain one '*'".fmt(f),
            Self::ParseIntError(error) => error.fmt(f),
        }
    }
}

impl From<ParseIntError> for ParseAreaError {
    fn from(error: ParseIntError) -> Self {
        Self::ParseIntError(error)
    }
}

#[allow(clippy::module_name_repetitions)]
pub fn try_parse_area(src: &str) -> Result<(u32, u32), ParseAreaError> {
    match src.find('*') {
        None => Ok((src.parse()?, 1)),
        Some(pos) => {
            if src[(pos + 1)..].find('*').is_some() {
                return Err(ParseAreaError::TooManyDimensions);
            }

            let first = src[..pos].trim().parse()?;
            let second = src[(pos + 1)..].trim().parse()?;

            Ok((first, second))
        },
    }
}
