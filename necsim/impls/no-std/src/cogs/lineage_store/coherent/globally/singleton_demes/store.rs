use core::marker::PhantomData;

use fnv::FnvBuildHasher;
use hashbrown::HashMap;
use slab::Slab;

use necsim_core::{
    cogs::{
        Backup, GloballyCoherentLineageStore, LineageStore, LocallyCoherentLineageStore, MathsCore,
    },
    landscape::{IndexedLocation, Location},
    lineage::{GlobalLineageReference, Lineage},
};

use crate::cogs::lineage_reference::in_memory::InMemoryLineageReference;

use super::{SingletonDemesHabitat, SingletonDemesLineageStore};

#[contract_trait]
impl<M: MathsCore, H: SingletonDemesHabitat<M>> LineageStore<M, H>
    for SingletonDemesLineageStore<M, H>
{
    type LocalLineageReference = InMemoryLineageReference;

    fn with_capacity(_habitat: &H, capacity: usize) -> Self {
        Self {
            lineages_store: Slab::with_capacity(capacity),
            location_to_lineage_reference: HashMap::with_capacity_and_hasher(
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
impl<M: MathsCore, H: SingletonDemesHabitat<M>> LocallyCoherentLineageStore<M, H>
    for SingletonDemesLineageStore<M, H>
{
    #[must_use]
    #[debug_requires(indexed_location.index() == 0, "only one lineage per location")]
    fn get_global_lineage_reference_at_indexed_location(
        &self,
        indexed_location: &IndexedLocation,
        _habitat: &H,
    ) -> Option<&GlobalLineageReference> {
        self.location_to_lineage_reference
            .get(indexed_location.location())
            .map(|local_reference| &self[local_reference].global_reference)
    }

    #[debug_requires(lineage.indexed_location.index() == 0, "only one lineage per location")]
    fn insert_lineage_locally_coherent(
        &mut self,
        lineage: Lineage,
        _habitat: &H,
    ) -> InMemoryLineageReference {
        let location = *lineage.indexed_location.location();

        // Safety: a new unique reference is issued here, no cloning occurs
        let local_lineage_reference =
            unsafe { InMemoryLineageReference::issue(self.lineages_store.insert(lineage)) };

        // Safety: the clone stays internal behind a reference
        self.location_to_lineage_reference.insert(location, unsafe {
            local_lineage_reference.backup_unchecked()
        });

        local_lineage_reference
    }

    #[must_use]
    #[debug_requires(
        self[&reference].indexed_location.index() == 0,
        "only one lineage per location"
    )]
    fn extract_lineage_locally_coherent(
        &mut self,
        reference: InMemoryLineageReference,
        _habitat: &H,
    ) -> Lineage {
        // We know from the trait preconditions that the lineage exists
        let lineage = self.lineages_store.remove(usize::from(reference));

        self.location_to_lineage_reference
            .remove(lineage.indexed_location.location());

        lineage
    }
}

#[contract_trait]
impl<M: MathsCore, H: SingletonDemesHabitat<M>> GloballyCoherentLineageStore<M, H>
    for SingletonDemesLineageStore<M, H>
{
    type LocationIterator<'a> = impl Iterator<Item = Location> + 'a where H: 'a;

    #[must_use]
    fn iter_active_locations(&self, _habitat: &H) -> Self::LocationIterator<'_> {
        self.lineages_store
            .iter()
            .map(|(_, lineage)| lineage.indexed_location.location())
            .copied()
    }

    #[must_use]
    fn get_local_lineage_references_at_location_unordered(
        &self,
        location: &Location,
        _habitat: &H,
    ) -> &[InMemoryLineageReference] {
        match self.location_to_lineage_reference.get(location) {
            Some(local_reference) => core::slice::from_ref(local_reference),
            None => &[],
        }
    }
}
