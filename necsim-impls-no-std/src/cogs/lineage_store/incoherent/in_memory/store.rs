use necsim_core::{
    cogs::{Habitat, IncoherentLineageStore, LineageStore},
    landscape::IndexedLocation,
    lineage::Lineage,
};

use crate::cogs::lineage_reference::in_memory::{
    InMemoryLineageReference, InMemoryLineageReferenceIterator,
};

use super::IncoherentInMemoryLineageStore;

#[contract_trait]
impl<H: Habitat> LineageStore<H, InMemoryLineageReference> for IncoherentInMemoryLineageStore<H> {
    type Iterator = InMemoryLineageReferenceIterator;

    #[must_use]
    fn new(sample_percentage: f64, habitat: &H) -> Self {
        Self::new_impl(sample_percentage, habitat)
    }

    #[must_use]
    fn get_number_total_lineages(&self) -> usize {
        self.lineages_store.len()
    }

    #[must_use]
    #[must_use]
    fn iter_local_lineage_references(&self) -> Self::Iterator {
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
impl<H: Habitat> IncoherentLineageStore<H, InMemoryLineageReference>
    for IncoherentInMemoryLineageStore<H>
{
    #[debug_requires(
        self.landscape_extent.contains(indexed_location.location()),
        "location is inside landscape extent"
    )]
    fn insert_lineage_to_indexed_location(
        &mut self,
        reference: InMemoryLineageReference,
        indexed_location: IndexedLocation,
    ) {
        unsafe {
            self.lineages_store[Into::<usize>::into(reference)]
                .move_to_indexed_location(indexed_location)
        }
    }

    #[must_use]
    #[debug_requires(
        self.landscape_extent.contains(self[reference].indexed_location().unwrap().location()),
        "lineage's location is inside landscape extent"
    )]
    fn extract_lineage_from_its_location(
        &mut self,
        reference: InMemoryLineageReference,
    ) -> IndexedLocation {
        unsafe { self.lineages_store[Into::<usize>::into(reference)].remove_from_location() }
    }
}
