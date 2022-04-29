use alloc::vec::Vec;
use core::marker::PhantomData;

use fnv::FnvBuildHasher;
use hashbrown::HashMap;
use slab::Slab;

use necsim_core::{
    cogs::{
        GloballyCoherentLineageStore, Habitat, LineageStore, LocallyCoherentLineageStore, MathsCore,
    },
    landscape::{IndexedLocation, Location},
    lineage::{GlobalLineageReference, Lineage},
};

use crate::cogs::lineage_reference::in_memory::InMemoryLineageReference;

use super::GillespieLineageStore;

#[contract_trait]
impl<M: MathsCore, H: Habitat<M>> LineageStore<M, H, InMemoryLineageReference>
    for GillespieLineageStore<M, H>
{
    #[allow(clippy::type_complexity)]
    type LineageReferenceIterator<'a>
    where
        H: 'a,
    = impl Iterator<Item = InMemoryLineageReference>;

    fn with_capacity(_habitat: &H, capacity: usize) -> Self {
        Self {
            lineages_store: Slab::with_capacity(capacity),
            location_to_lineage_references: HashMap::with_hasher(FnvBuildHasher::default()),
            indexed_location_to_lineage_reference: HashMap::with_capacity_and_hasher(
                capacity,
                FnvBuildHasher::default(),
            ),
            _marker: PhantomData::<(M, H)>,
        }
    }

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
impl<M: MathsCore, H: Habitat<M>> LocallyCoherentLineageStore<M, H, InMemoryLineageReference>
    for GillespieLineageStore<M, H>
{
    #[must_use]
    fn get_global_lineage_reference_at_indexed_location(
        &self,
        indexed_location: &IndexedLocation,
        _habitat: &H,
    ) -> Option<&GlobalLineageReference> {
        self.indexed_location_to_lineage_reference
            .get(indexed_location)
            .map(|(global_reference, _index)| global_reference)
    }

    fn insert_lineage_locally_coherent(
        &mut self,
        lineage: Lineage,
        _habitat: &H,
    ) -> InMemoryLineageReference {
        let lineages_at_location = self
            .location_to_lineage_references
            .entry(lineage.indexed_location.location().clone())
            .or_insert(Vec::new());

        self.indexed_location_to_lineage_reference.insert(
            lineage.indexed_location.clone(),
            (lineage.global_reference.clone(), lineages_at_location.len()),
        );

        let local_lineage_reference =
            InMemoryLineageReference::from(self.lineages_store.insert(lineage));

        lineages_at_location.push(local_lineage_reference);

        local_lineage_reference
    }

    #[must_use]
    fn extract_lineage_locally_coherent(
        &mut self,
        reference: InMemoryLineageReference,
        _habitat: &H,
    ) -> Lineage {
        let lineage = self.lineages_store.remove(usize::from(reference));

        // We know from the trait preconditions that this value exists
        let (_global_reference, local_index) = self
            .indexed_location_to_lineage_reference
            .remove(&lineage.indexed_location)
            .unwrap();

        // We know from the integrity of this store that this value exists
        let lineages_at_location = self
            .location_to_lineage_references
            .get_mut(lineage.indexed_location.location())
            .unwrap();

        lineages_at_location.swap_remove(local_index);

        // Only executed if reference was not the last item in lineages_at_location
        if let Some(replacement_local_reference) = lineages_at_location.get(local_index) {
            let replacement_location =
                &self.lineages_store[usize::from(*replacement_local_reference)].indexed_location;

            // Always executed since the replacement lineage is active
            if let Some((_replacement_global_reference, replacement_index)) = self
                .indexed_location_to_lineage_reference
                .get_mut(replacement_location)
            {
                *replacement_index = local_index;
            }
        }

        lineage
    }
}

#[contract_trait]
impl<M: MathsCore, H: Habitat<M>> GloballyCoherentLineageStore<M, H, InMemoryLineageReference>
    for GillespieLineageStore<M, H>
{
    type LocationIterator<'a>
    where
        H: 'a,
    = impl Iterator<Item = Location>;

    #[must_use]
    fn iter_active_locations(&self, _habitat: &H) -> Self::LocationIterator<'_> {
        self.location_to_lineage_references
            .iter()
            .filter_map(|(location, references)| {
                if references.is_empty() {
                    None
                } else {
                    Some(location.clone())
                }
            })
    }

    #[must_use]
    fn get_local_lineage_references_at_location_unordered(
        &self,
        location: &Location,
        _habitat: &H,
    ) -> &[InMemoryLineageReference] {
        self.location_to_lineage_references
            .get(location)
            .map_or(&[], |references| references)
    }
}
