use core::{fmt, iter::Iterator};

use necsim_core::{
    cogs::OriginSampler,
    intrinsics::ceil,
    landscape::{IndexedLocation, LandscapeExtent, LocationIterator},
};

use crate::cogs::{
    habitat::almost_infinite::AlmostInfiniteHabitat, origin_sampler::pre_sampler::OriginPreSampler,
};

const HABITAT_CENTRE: u32 = u32::MAX / 2;

#[allow(clippy::module_name_repetitions)]
pub struct AlmostInfiniteOriginSampler<'h, I: Iterator<Item = u64>> {
    pre_sampler: OriginPreSampler<I>,
    last_index: u64,
    location_iterator: LocationIterator,
    radius_squared: u64,
    upper_bound_size_hint: u64,
    habitat: &'h AlmostInfiniteHabitat,
}

impl<'h, I: Iterator<Item = u64>> fmt::Debug for AlmostInfiniteOriginSampler<'h, I> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AlmostInfiniteOriginSampler")
            .field("pre_sampler", &self.pre_sampler)
            .field("last_index", &self.last_index)
            .field("location_iterator", &self.location_iterator)
            .field("radius_squared", &self.radius_squared)
            .field("upper_bound_size_hint", &self.upper_bound_size_hint)
            .field("habitat", &self.habitat)
            .finish()
    }
}

impl<'h, I: Iterator<Item = u64>> AlmostInfiniteOriginSampler<'h, I> {
    #[debug_requires(radius < (u32::MAX / 2), "sample circle fits into almost infinite habitat")]
    #[must_use]
    pub fn new(
        pre_sampler: OriginPreSampler<I>,
        habitat: &'h AlmostInfiniteHabitat,
        radius: u32,
    ) -> Self {
        let sample_extent = LandscapeExtent::new(
            HABITAT_CENTRE - radius,
            HABITAT_CENTRE - radius,
            radius * 2 + 1,
            radius * 2 + 1,
        );

        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let upper_bound_size_hint = ceil(
            f64::from(radius)
                * f64::from(radius)
                * core::f64::consts::PI
                * pre_sampler.get_sample_proportion(),
        ) as u64;

        Self {
            pre_sampler,
            last_index: 0_u64,
            location_iterator: sample_extent.iter(),
            radius_squared: u64::from(radius) * u64::from(radius),
            upper_bound_size_hint,
            habitat,
        }
    }
}

#[contract_trait]
impl<'h, I: Iterator<Item = u64>> OriginSampler<'h> for AlmostInfiniteOriginSampler<'h, I> {
    type Habitat = AlmostInfiniteHabitat;

    fn habitat(&self) -> &'h Self::Habitat {
        self.habitat
    }

    fn full_upper_bound_size_hint(&self) -> u64 {
        self.upper_bound_size_hint
    }
}

impl<'h, I: Iterator<Item = u64>> Iterator for AlmostInfiniteOriginSampler<'h, I> {
    type Item = IndexedLocation;

    fn next(&mut self) -> Option<Self::Item> {
        let next_index = self.pre_sampler.next()?;
        let mut index_difference = (next_index - self.last_index).saturating_sub(1);
        self.last_index = next_index;

        while let Some(next_location) = self.location_iterator.next() {
            let dx = i64::from(next_location.x()) - i64::from(HABITAT_CENTRE);
            let dy = i64::from(next_location.y()) - i64::from(HABITAT_CENTRE);

            #[allow(clippy::cast_sign_loss)]
            let distance_squared = (dx * dx) as u64 + (dy * dy) as u64;

            if distance_squared <= self.radius_squared {
                if index_difference == 0 {
                    return Some(IndexedLocation::new(next_location, 0));
                }

                index_difference -= 1;
            }
        }

        None
    }
}
