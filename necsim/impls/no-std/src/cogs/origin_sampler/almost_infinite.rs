use core::iter::Iterator;

use necsim_core::{
    cogs::OriginSampler,
    intrinsics::ceil,
    landscape::{IndexedLocation, LandscapeExtent, LocationIterator},
};

use crate::cogs::habitat::almost_infinite::AlmostInfiniteHabitat;

const HABITAT_CENTRE: u32 = u32::MAX / 2;

#[allow(clippy::module_name_repetitions)]
#[derive(Debug)]
pub struct AlmostInfiniteOriginSampler<'h> {
    location_iterator: LocationIterator,
    radius_squared: u64,
    upper_bound_size_hint: u64,
    habitat: &'h AlmostInfiniteHabitat,
}

impl<'h> AlmostInfiniteOriginSampler<'h> {
    #[debug_requires(radius < (u32::MAX / 2), "sample circle fits into almost infinite habitat")]
    #[must_use]
    pub fn new(habitat: &'h AlmostInfiniteHabitat, radius: u32) -> Self {
        let sample_extent = LandscapeExtent::new(
            HABITAT_CENTRE - radius,
            HABITAT_CENTRE - radius,
            radius * 2 + 1,
            radius * 2 + 1,
        );

        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let upper_bound_size_hint =
            ceil(f64::from(radius) * f64::from(radius) * core::f64::consts::PI) as u64;

        Self {
            location_iterator: sample_extent.iter(),
            radius_squared: u64::from(radius) * u64::from(radius),
            upper_bound_size_hint,
            habitat,
        }
    }
}

#[contract_trait]
impl<'h> OriginSampler<'h, AlmostInfiniteHabitat> for AlmostInfiniteOriginSampler<'h> {
    fn habitat(&self) -> &'h AlmostInfiniteHabitat {
        self.habitat
    }

    fn full_upper_bound_size_hint(&self) -> u64 {
        self.upper_bound_size_hint
    }
}

impl<'h> Iterator for AlmostInfiniteOriginSampler<'h> {
    type Item = IndexedLocation;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(next_location) = self.location_iterator.next() {
            let dx = i64::from(next_location.x()) - i64::from(HABITAT_CENTRE);
            let dy = i64::from(next_location.y()) - i64::from(HABITAT_CENTRE);

            #[allow(clippy::cast_sign_loss)]
            let distance_squared = (dx * dx) as u64 + (dy * dy) as u64;

            if distance_squared <= self.radius_squared {
                return Some(IndexedLocation::new(next_location, 0));
            }
        }

        None
    }
}
