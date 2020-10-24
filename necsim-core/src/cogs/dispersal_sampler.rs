use crate::landscape::Location;
use crate::rng::Rng;

use super::Habitat;

pub trait DispersalSampler<H: Habitat> {
    #[must_use]
    fn sample_dispersal_from_location(&self, location: &Location, rng: &mut impl Rng) -> Location;
}
