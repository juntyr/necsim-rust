mod extent;
mod location;

#[allow(clippy::useless_attribute, clippy::module_name_repetitions)]
pub use extent::{LandscapeExtent, LocationIterator};
pub use location::{IndexedLocation, Location};
