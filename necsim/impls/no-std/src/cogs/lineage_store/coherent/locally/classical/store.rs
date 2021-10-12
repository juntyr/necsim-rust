use necsim_core::{
    cogs::{Habitat, LineageStore, LocallyCoherentLineageStore, OriginSampler, F64Core},
    landscape::IndexedLocation,
    lineage::{GlobalLineageReference, Lineage},
};

use crate::cogs::lineage_reference::in_memory::InMemoryLineageReference;

use super::ClassicalLineageStore;

#[contract_trait]
impl<F: F64Core, H: Habitat<F>> LineageStore<F, H, InMemoryLineageReference> for ClassicalLineageStore<F, H> {
    type LineageReferenceIterator<'a> = impl Iterator<Item = InMemoryLineageReference>;

    fn from_origin_sampler<'h, O: OriginSampler<'h, F, Habitat = H>>(origin_sampler: O) -> Self
    where
        H: 'h,
    {
        Self::new(origin_sampler)
    }

    #[must_use]
    #[must_use]
    fn iter_local_lineage_references(&self) -> Self::LineageReferenceIterator<'_> {
        self.lineages_store
            .iter()
            .map(|(reference, _)| InMemoryLineageReference::from(reference))
    }

    #[must_use]
    fn get_lineage_for_local_reference(
        &self,
        reference: InMemoryLineageReference,
    ) -> Option<&Lineage> {
        self.lineages_store.get(usize::from(reference))
    }
}

#[contract_trait]
impl<F: F64Core, H: Habitat<F>> LocallyCoherentLineageStore<F, H, InMemoryLineageReference>
    for ClassicalLineageStore<F, H>
{
    #[must_use]
    fn get_global_lineage_reference_at_indexed_location(
        &self,
        indexed_location: &IndexedLocation,
        _habitat: &H,
    ) -> Option<&GlobalLineageReference> {
        self.indexed_location_to_lineage_reference
            .get(indexed_location)
            .map(|local_reference| &self[*local_reference].global_reference)
    }

    fn insert_lineage_locally_coherent(
        &mut self,
        lineage: Lineage,
        _habitat: &H,
    ) -> InMemoryLineageReference {
        let indexed_location = lineage.indexed_location.clone();

        let local_lineage_reference =
            InMemoryLineageReference::from(self.lineages_store.insert(lineage));

        self.indexed_location_to_lineage_reference
            .insert(indexed_location, local_lineage_reference);

        local_lineage_reference
    }

    #[must_use]
    fn extract_lineage_locally_coherent(
        &mut self,
        reference: InMemoryLineageReference,
        _habitat: &H,
    ) -> Lineage {
        // We know from the trait preconditions that the lineage exists
        let lineage = self.lineages_store.remove(usize::from(reference));

        self.indexed_location_to_lineage_reference
            .remove(&lineage.indexed_location);

        lineage
    }
}
