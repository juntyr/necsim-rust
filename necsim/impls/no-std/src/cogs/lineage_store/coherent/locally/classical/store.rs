use necsim_core::{
    cogs::{Habitat, LineageStore, LocallyCoherentLineageStore, OriginSampler},
    landscape::IndexedLocation,
    lineage::{GlobalLineageReference, Lineage},
};
use necsim_core_bond::{NonNegativeF64, PositiveF64};

use crate::cogs::lineage_reference::in_memory::InMemoryLineageReference;

use super::ClassicalLineageStore;

#[contract_trait]
impl<H: Habitat> LineageStore<H, InMemoryLineageReference> for ClassicalLineageStore<H> {
    #[allow(clippy::type_complexity)]
    type LineageReferenceIterator<'a> = core::iter::Map<
        slab::Iter<'a, Lineage>,
        fn((usize, &'a Lineage)) -> InMemoryLineageReference,
    >;

    fn from_origin_sampler<'h, O: OriginSampler<'h, Habitat = H>>(origin_sampler: O) -> Self
    where
        H: 'h,
    {
        Self::new(origin_sampler)
    }

    #[must_use]
    fn get_number_total_lineages(&self) -> usize {
        self.lineages_store.len()
    }

    #[must_use]
    #[must_use]
    fn iter_local_lineage_references(&self) -> Self::LineageReferenceIterator<'_> {
        self.lineages_store.iter().map(
            (|(reference, _)| InMemoryLineageReference::from(reference))
                as fn((usize, &'_ Lineage)) -> InMemoryLineageReference,
        )
    }

    #[must_use]
    fn get(&self, reference: InMemoryLineageReference) -> Option<&Lineage> {
        self.lineages_store.get(usize::from(reference))
    }
}

#[contract_trait]
impl<H: Habitat> LocallyCoherentLineageStore<H, InMemoryLineageReference>
    for ClassicalLineageStore<H>
{
    #[must_use]
    fn get_active_global_lineage_reference_at_indexed_location(
        &self,
        indexed_location: &IndexedLocation,
        _habitat: &H,
    ) -> Option<&GlobalLineageReference> {
        self.indexed_location_to_lineage_reference
            .get(indexed_location)
            .map(|local_reference| self[*local_reference].global_reference())
    }

    fn insert_lineage_to_indexed_location_locally_coherent(
        &mut self,
        reference: InMemoryLineageReference,
        indexed_location: IndexedLocation,
        _habitat: &H,
    ) {
        self.indexed_location_to_lineage_reference
            .insert(indexed_location.clone(), reference);

        unsafe {
            self.lineages_store[usize::from(reference)].move_to_indexed_location(indexed_location);
        };
    }

    #[must_use]
    fn extract_lineage_from_its_location_locally_coherent(
        &mut self,
        reference: InMemoryLineageReference,
        event_time: PositiveF64,
        _habitat: &H,
    ) -> (IndexedLocation, NonNegativeF64) {
        let (lineage_indexed_location, prior_time) =
            unsafe { self.lineages_store[usize::from(reference)].remove_from_location(event_time) };

        // We know from the trait preconditions that this value exists
        let _global_reference = self
            .indexed_location_to_lineage_reference
            .remove(&lineage_indexed_location);

        (lineage_indexed_location, prior_time)
    }

    fn emigrate(
        &mut self,
        local_lineage_reference: InMemoryLineageReference,
    ) -> GlobalLineageReference {
        self.lineages_store
            .remove(local_lineage_reference.into())
            .emigrate()
    }

    fn immigrate_locally_coherent(
        &mut self,
        _habitat: &H,
        global_reference: GlobalLineageReference,
        indexed_location: IndexedLocation,
        time_of_emigration: PositiveF64,
    ) -> InMemoryLineageReference {
        let lineage = Lineage::immigrate(
            global_reference,
            indexed_location.clone(),
            time_of_emigration,
        );

        let local_lineage_reference =
            InMemoryLineageReference::from(self.lineages_store.insert(lineage));

        self.indexed_location_to_lineage_reference
            .insert(indexed_location, local_lineage_reference);

        local_lineage_reference
    }
}
