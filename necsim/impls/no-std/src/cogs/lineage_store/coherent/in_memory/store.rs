use necsim_core::{
    cogs::{CoherentLineageStore, Habitat, LineageStore},
    landscape::{IndexedLocation, Location, LocationIterator},
    lineage::Lineage,
};

use crate::cogs::lineage_reference::in_memory::{
    InMemoryLineageReference, InMemoryLineageReferenceIterator,
};

use super::CoherentInMemoryLineageStore;

#[contract_trait]
impl<H: Habitat> LineageStore<H, InMemoryLineageReference> for CoherentInMemoryLineageStore<H> {
    type LineageReferenceIterator<'a> = InMemoryLineageReferenceIterator;

    #[must_use]
    fn get_number_total_lineages(&self) -> usize {
        self.lineages_store.len()
    }

    #[must_use]
    #[must_use]
    fn iter_local_lineage_references(&self) -> Self::LineageReferenceIterator<'_> {
        InMemoryLineageReferenceIterator::from(self.lineages_store.len())
    }

    #[must_use]
    fn get(&self, reference: InMemoryLineageReference) -> Option<&Lineage> {
        self.lineages_store.get(Into::<usize>::into(reference))
    }

    fn update_lineage_time_of_last_event(
        &mut self,
        reference: InMemoryLineageReference,
        event_time: f64,
    ) {
        unsafe {
            self.lineages_store[Into::<usize>::into(reference)]
                .update_time_of_last_event(event_time)
        }
    }
}

#[contract_trait]
impl<H: Habitat> CoherentLineageStore<H, InMemoryLineageReference>
    for CoherentInMemoryLineageStore<H>
{
    type LocationIterator<'a> = LocationIterator;

    #[must_use]
    fn iter_active_locations(&self) -> Self::LocationIterator<'_> {
        self.landscape_extent.iter()
    }

    #[must_use]
    #[debug_requires(
        self.landscape_extent.contains(location),
        "location is inside landscape extent"
    )]
    fn get_active_lineages_at_location(&self, location: &Location) -> &[InMemoryLineageReference] {
        &self.location_to_lineage_references[(
            (location.y() - self.landscape_extent.y()) as usize,
            (location.x() - self.landscape_extent.x()) as usize,
        )]
    }

    #[debug_requires(
        self.landscape_extent.contains(&location),
        "location is inside landscape extent"
    )]
    fn append_lineage_to_location(
        &mut self,
        reference: InMemoryLineageReference,
        location: Location,
    ) {
        let lineages_at_location = &mut self.location_to_lineage_references[(
            (location.y() - self.landscape_extent.y()) as usize,
            (location.x() - self.landscape_extent.x()) as usize,
        )];

        lineages_at_location.push(reference);

        #[allow(clippy::cast_possible_truncation)]
        let new_indexed_location =
            IndexedLocation::new(location, (lineages_at_location.len() - 1) as u32);

        unsafe {
            self.lineages_store[Into::<usize>::into(reference)]
                .move_to_indexed_location(new_indexed_location)
        };
    }

    #[must_use]
    #[debug_requires(
        self.landscape_extent.contains(self[reference].indexed_location().unwrap().location()),
        "lineage's location is inside landscape extent"
    )]
    fn pop_lineage_from_its_location(
        &mut self,
        reference: InMemoryLineageReference,
    ) -> IndexedLocation {
        let lineage: &Lineage = &self.lineages_store[Into::<usize>::into(reference)];

        let lineage_indexed_location = lineage.indexed_location().unwrap();

        let lineage_location = lineage_indexed_location.location();
        let lineage_index_at_location = lineage_indexed_location.index();

        let lineages_at_location = &mut self.location_to_lineage_references[(
            (lineage_location.y() - self.landscape_extent.y()) as usize,
            (lineage_location.x() - self.landscape_extent.x()) as usize,
        )];

        if let Some(last_lineage_at_location) = lineages_at_location.pop() {
            if (lineage_index_at_location as usize) < lineages_at_location.len() {
                lineages_at_location[lineage_index_at_location as usize] = last_lineage_at_location;

                unsafe {
                    self.lineages_store[Into::<usize>::into(last_lineage_at_location)]
                        .update_index_at_location(lineage_index_at_location)
                };
            }

            unsafe { self.lineages_store[Into::<usize>::into(reference)].remove_from_location() }
        } else {
            unreachable!()
        }
    }
}
