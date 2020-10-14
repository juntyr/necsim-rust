#[allow(clippy::module_name_repetitions)]
pub struct NewGdalError(gdal::errors::Error);

impl std::fmt::Debug for NewGdalError {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(&self.0, fmt)
    }
}

impl std::fmt::Display for NewGdalError {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&self.0, fmt)
    }
}

impl std::error::Error for NewGdalError {}

impl From<gdal::errors::Error> for NewGdalError {
    fn from(error: gdal::errors::Error) -> Self {
        Self(error)
    }
}
