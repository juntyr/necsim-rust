use necsim_core::{
    cogs::{IncoherentLineageStore, LineageStore},
    landscape::IndexedLocation,
    lineage::Lineage,
};

use crate::cogs::lineage_reference::in_memory::{
    InMemoryLineageReference, InMemoryLineageReferenceIterator,
};

use crate::cogs::habitat::almost_infinite::AlmostInfiniteHabitat;

use super::IncoherentAlmostInfiniteLineageStore;

#[contract_trait]
impl LineageStore<AlmostInfiniteHabitat, InMemoryLineageReference>
    for IncoherentAlmostInfiniteLineageStore
{
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
impl IncoherentLineageStore<AlmostInfiniteHabitat, InMemoryLineageReference>
    for IncoherentAlmostInfiniteLineageStore
{
    fn insert_lineage_to_indexed_location(
        &mut self,
        reference: InMemoryLineageReference,
        indexed_location: IndexedLocation,
        _habitat: &AlmostInfiniteHabitat,
    ) {
        unsafe {
            self.lineages_store[Into::<usize>::into(reference)]
                .move_to_indexed_location(indexed_location)
        }
    }

    #[must_use]
    fn extract_lineage_from_its_location(
        &mut self,
        reference: InMemoryLineageReference,
        _habitat: &AlmostInfiniteHabitat,
    ) -> IndexedLocation {
        unsafe { self.lineages_store[Into::<usize>::into(reference)].remove_from_location() }
    }
}
