use core::iter::{Iterator, Peekable};

use necsim_core::{
    cogs::{Habitat, OriginSampler},
    landscape::{IndexedLocation, LocationIterator},
};

use crate::cogs::habitat::non_spatial::NonSpatialHabitat;

#[allow(clippy::module_name_repetitions)]
#[derive(Debug)]
pub struct NonSpatialOriginSampler<'h> {
    location_iterator: Peekable<LocationIterator>,
    next_index: u32,
    habitat: &'h NonSpatialHabitat,
}

impl<'h> NonSpatialOriginSampler<'h> {
    #[must_use]
    pub fn new(habitat: &'h NonSpatialHabitat) -> Self {
        Self {
            location_iterator: habitat.get_extent().iter().peekable(),
            next_index: 0_u32,
            habitat,
        }
    }
}

#[contract_trait]
impl<'h> OriginSampler<'h, NonSpatialHabitat> for NonSpatialOriginSampler<'h> {
    fn habitat(&self) -> &'h NonSpatialHabitat {
        self.habitat
    }

    fn full_upper_bound_size_hint(&self) -> u64 {
        self.habitat.get_total_habitat()
    }
}

impl<'h> Iterator for NonSpatialOriginSampler<'h> {
    type Item = IndexedLocation;

    fn next(&mut self) -> Option<Self::Item> {
        if self.habitat.get_deme() == 0 {
            return None;
        }

        if self.next_index >= self.habitat.get_deme() {
            self.next_index = 0;
            self.location_iterator.next();
        }

        let next_location = self.location_iterator.peek()?;
        let next_index = self.next_index;

        self.next_index += 1;

        Some(IndexedLocation::new(next_location.clone(), next_index))
    }
}
