use core::{
    convert::TryFrom,
    fmt,
    iter::{Iterator, Peekable},
};

use necsim_core::{
    cogs::{Habitat, OriginSampler},
    landscape::{IndexedLocation, LocationIterator},
};

use crate::cogs::{
    habitat::non_spatial::NonSpatialHabitat, origin_sampler::pre_sampler::OriginPreSampler,
};

#[allow(clippy::module_name_repetitions)]
pub struct NonSpatialOriginSampler<'h, I: Iterator<Item = u64>> {
    pre_sampler: OriginPreSampler<I>,
    last_index: u64,
    location_iterator: Peekable<LocationIterator>,
    next_location_index: u32,
    habitat: &'h NonSpatialHabitat,
}

impl<'h, I: Iterator<Item = u64>> fmt::Debug for NonSpatialOriginSampler<'h, I> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("NonSpatialOriginSampler")
            .field("pre_sampler", &self.pre_sampler)
            .field("last_index", &self.last_index)
            .field("location_iterator", &self.location_iterator)
            .field("next_location_index", &self.next_location_index)
            .field("habitat", &self.habitat)
            .finish()
    }
}

impl<'h, I: Iterator<Item = u64>> NonSpatialOriginSampler<'h, I> {
    #[must_use]
    pub fn new(pre_sampler: OriginPreSampler<I>, habitat: &'h NonSpatialHabitat) -> Self {
        Self {
            pre_sampler,
            last_index: 0_u64,
            location_iterator: habitat.get_extent().iter().peekable(),
            next_location_index: 0_u32,
            habitat,
        }
    }
}

#[contract_trait]
impl<'h, I: Iterator<Item = u64>> OriginSampler<'h> for NonSpatialOriginSampler<'h, I> {
    type Habitat = NonSpatialHabitat;

    fn habitat(&self) -> &'h Self::Habitat {
        self.habitat
    }

    fn full_upper_bound_size_hint(&self) -> u64 {
        #[allow(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            clippy::cast_precision_loss
        )]
        {
            ((self.habitat.get_total_habitat() as f64) * self.pre_sampler.get_sample_proportion())
                as u64
        }
    }
}

impl<'h, I: Iterator<Item = u64>> Iterator for NonSpatialOriginSampler<'h, I> {
    type Item = IndexedLocation;

    fn next(&mut self) -> Option<Self::Item> {
        if self.habitat.get_deme() == 0 {
            return None;
        }

        let next_index = self.pre_sampler.next()?;
        let mut index_difference = next_index - self.last_index;
        self.last_index = next_index;

        while u64::from(self.next_location_index) + index_difference
            >= u64::from(self.habitat.get_deme())
        {
            index_difference -= u64::from(self.habitat.get_deme());

            self.next_location_index = 0;

            self.location_iterator.next();
        }

        let next_location = self.location_iterator.peek()?;

        self.next_location_index += u32::try_from(index_difference).unwrap();

        Some(IndexedLocation::new(
            next_location.clone(),
            self.next_location_index,
        ))
    }
}
