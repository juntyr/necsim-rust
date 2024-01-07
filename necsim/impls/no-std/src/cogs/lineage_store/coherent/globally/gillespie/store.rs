use core::marker::PhantomData;

use fnv::FnvBuildHasher;
use hashbrown::HashMap;
use slab::Slab;

use necsim_core::{
    cogs::{
        Backup, GloballyCoherentLineageStore, Habitat, LineageStore, LocallyCoherentLineageStore,
        MathsCore,
    },
    landscape::{IndexedLocation, Location},
    lineage::{GlobalLineageReference, Lineage},
};

use crate::cogs::lineage_reference::in_memory::InMemoryLineageReference;

use super::GillespieLineageStore;

#[contract_trait]
impl<M: MathsCore, H: Habitat<M>> LineageStore<M, H> for GillespieLineageStore<M, H> {
    type LocalLineageReference = InMemoryLineageReference;

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
    fn get_lineage_for_local_reference(
        &self,
        reference: &InMemoryLineageReference,
    ) -> Option<&Lineage> {
        self.lineages_store.get(usize::from(reference))
    }
}

#[contract_trait]
impl<M: MathsCore, H: Habitat<M>> LocallyCoherentLineageStore<M, H>
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
            .entry(*lineage.indexed_location.location())
            .or_default();

        self.indexed_location_to_lineage_reference.insert(
            lineage.indexed_location,
            (lineage.global_reference.clone(), lineages_at_location.len()),
        );

        // Safety: a new unique reference is issued here, no cloning occurs
        let local_lineage_reference =
            unsafe { InMemoryLineageReference::issue(self.lineages_store.insert(lineage)) };

        // Safety: the clone stays internal behind a reference
        lineages_at_location.push(unsafe { local_lineage_reference.backup_unchecked() });

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
                &self.lineages_store[usize::from(replacement_local_reference)].indexed_location;

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
impl<M: MathsCore, H: Habitat<M>> GloballyCoherentLineageStore<M, H>
    for GillespieLineageStore<M, H>
{
    type LocationIterator<'a> = impl Iterator<Item = Location> + 'a where H: 'a;

    #[must_use]
    fn iter_active_locations(&self, _habitat: &H) -> Self::LocationIterator<'_> {
        self.location_to_lineage_references
            .iter()
            .filter_map(|(location, references)| {
                if references.is_empty() {
                    None
                } else {
                    Some(*location)
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
