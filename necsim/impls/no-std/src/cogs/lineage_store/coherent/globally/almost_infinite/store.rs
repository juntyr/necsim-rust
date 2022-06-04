use core::marker::PhantomData;

use fnv::FnvBuildHasher;
use hashbrown::HashMap;
use slab::Slab;

use necsim_core::{
    cogs::{GloballyCoherentLineageStore, LineageStore, LocallyCoherentLineageStore, MathsCore},
    landscape::{IndexedLocation, Location},
    lineage::{GlobalLineageReference, Lineage},
};

use crate::cogs::{
    habitat::almost_infinite::AlmostInfiniteHabitat,
    lineage_reference::in_memory::InMemoryLineageReference,
};

use super::AlmostInfiniteLineageStore;

#[contract_trait]
impl<M: MathsCore> LineageStore<M, AlmostInfiniteHabitat<M>> for AlmostInfiniteLineageStore<M> {
    type LocalLineageReference = InMemoryLineageReference;

    fn with_capacity(_habitat: &AlmostInfiniteHabitat<M>, capacity: usize) -> Self {
        Self {
            lineages_store: Slab::with_capacity(capacity),
            location_to_lineage_reference: HashMap::with_capacity_and_hasher(
                capacity,
                FnvBuildHasher::default(),
            ),
            _marker: PhantomData::<M>,
        }
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
impl<M: MathsCore> LocallyCoherentLineageStore<M, AlmostInfiniteHabitat<M>>
    for AlmostInfiniteLineageStore<M>
{
    #[must_use]
    #[debug_requires(indexed_location.index() == 0, "only one lineage per location")]
    fn get_global_lineage_reference_at_indexed_location(
        &self,
        indexed_location: &IndexedLocation,
        _habitat: &AlmostInfiniteHabitat<M>,
    ) -> Option<&GlobalLineageReference> {
        self.location_to_lineage_reference
            .get(indexed_location.location())
            .map(|local_reference| &self[*local_reference].global_reference)
    }

    #[debug_requires(lineage.indexed_location.index() == 0, "only one lineage per location")]
    fn insert_lineage_locally_coherent(
        &mut self,
        lineage: Lineage,
        _habitat: &AlmostInfiniteHabitat<M>,
    ) -> InMemoryLineageReference {
        let location = lineage.indexed_location.location().clone();

        let local_lineage_reference =
            InMemoryLineageReference::from(self.lineages_store.insert(lineage));

        self.location_to_lineage_reference
            .insert(location, local_lineage_reference);

        local_lineage_reference
    }

    #[must_use]
    #[debug_requires(
        self[reference].indexed_location.index() == 0,
        "only one lineage per location"
    )]
    fn extract_lineage_locally_coherent(
        &mut self,
        reference: InMemoryLineageReference,
        _habitat: &AlmostInfiniteHabitat<M>,
    ) -> Lineage {
        // We know from the trait preconditions that the lineage exists
        let lineage = self.lineages_store.remove(usize::from(reference));

        self.location_to_lineage_reference
            .remove(lineage.indexed_location.location());

        lineage
    }
}

#[contract_trait]
impl<M: MathsCore> GloballyCoherentLineageStore<M, AlmostInfiniteHabitat<M>>
    for AlmostInfiniteLineageStore<M>
{
    type LocationIterator<'a> = impl Iterator<Item = Location>;

    #[must_use]
    fn iter_active_locations(
        &self,
        _habitat: &AlmostInfiniteHabitat<M>,
    ) -> Self::LocationIterator<'_> {
        self.lineages_store
            .iter()
            .map(|(_, lineage)| lineage.indexed_location.location())
            .cloned()
    }

    #[must_use]
    fn get_local_lineage_references_at_location_unordered(
        &self,
        location: &Location,
        _habitat: &AlmostInfiniteHabitat<M>,
    ) -> &[InMemoryLineageReference] {
        match self.location_to_lineage_reference.get(location) {
            Some(local_reference) => core::slice::from_ref(local_reference),
            None => &[],
        }
    }
}
