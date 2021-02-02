use core::iter::{Iterator, Peekable};

use necsim_core::{
    cogs::{Habitat, OriginSampler},
    landscape::{IndexedLocation, LocationIterator},
};

use crate::cogs::habitat::in_memory::InMemoryHabitat;

#[allow(clippy::module_name_repetitions)]
#[derive(Debug)]
pub struct InMemoryOriginSampler<'h> {
    location_iterator: Peekable<LocationIterator>,
    next_index: u32,
    habitat: &'h InMemoryHabitat,
}

impl<'h> InMemoryOriginSampler<'h> {
    #[must_use]
    pub fn new(habitat: &'h InMemoryHabitat) -> Self {
        Self {
            location_iterator: habitat.get_extent().iter().peekable(),
            next_index: 0_u32,
            habitat,
        }
    }
}

#[contract_trait]
impl<'h> OriginSampler<'h, InMemoryHabitat> for InMemoryOriginSampler<'h> {
    fn habitat(&self) -> &'h InMemoryHabitat {
        self.habitat
    }

    fn full_upper_bound_size_hint(&self) -> u64 {
        self.habitat.get_total_habitat()
    }
}

impl<'h> Iterator for InMemoryOriginSampler<'h> {
    type Item = IndexedLocation;

    fn next(&mut self) -> Option<Self::Item> {
        while self.next_index
            >= self
                .habitat
                .get_habitat_at_location(self.location_iterator.peek()?)
        {
            self.next_index = 0;

            self.location_iterator.next();
        }

        let next_location = self.location_iterator.peek()?;
        let next_index = self.next_index;

        self.next_index += 1;

        Some(IndexedLocation::new(next_location.clone(), next_index))
    }
}
