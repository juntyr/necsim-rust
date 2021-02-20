use necsim_core::{
    cogs::{CoherentLineageStore, LineageStore},
    landscape::{IndexedLocation, Location},
    lineage::{GlobalLineageReference, Lineage},
};

use crate::cogs::lineage_reference::in_memory::{
    InMemoryLineageReference, InMemoryLineageReferenceIterator,
};

use crate::cogs::habitat::almost_infinite::AlmostInfiniteHabitat;

use super::CoherentAlmostInfiniteLineageStore;

#[contract_trait]
impl LineageStore<AlmostInfiniteHabitat, InMemoryLineageReference>
    for CoherentAlmostInfiniteLineageStore
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
}

#[contract_trait]
impl CoherentLineageStore<AlmostInfiniteHabitat, InMemoryLineageReference>
    for CoherentAlmostInfiniteLineageStore
{
    #[allow(clippy::type_complexity)]
    type LocationIterator<'a> = core::iter::Cloned<
        core::iter::FilterMap<
            core::slice::Iter<'a, Lineage>,
            for<'l> fn(&'l Lineage) -> Option<&'l necsim_core::landscape::Location>,
        >,
    >;

    #[must_use]
    fn iter_active_locations(
        &self,
        _habitat: &AlmostInfiniteHabitat,
    ) -> Self::LocationIterator<'_> {
        self.lineages_store
            .iter()
            .filter_map(
                (|lineage| lineage.indexed_location().map(IndexedLocation::location))
                    as for<'l> fn(&'l Lineage) -> Option<&'l Location>,
            )
            .cloned()
    }

    #[must_use]
    fn get_active_local_lineage_references_at_location_unordered(
        &self,
        location: &Location,
        _habitat: &AlmostInfiniteHabitat,
    ) -> &[InMemoryLineageReference] {
        match self.location_to_lineage_references.get(location) {
            Some(local_reference) => core::slice::from_ref(local_reference),
            None => &[],
        }
    }

    #[must_use]
    #[debug_requires(indexed_location.index() == 0, "only one lineage per location")]
    fn get_active_global_lineage_reference_at_indexed_location(
        &self,
        indexed_location: &IndexedLocation,
        _habitat: &AlmostInfiniteHabitat,
    ) -> Option<&GlobalLineageReference> {
        self.location_to_lineage_references
            .get(indexed_location.location())
            .map(|local_reference| self[*local_reference].global_reference())
    }

    #[debug_requires(indexed_location.index() == 0, "only one lineage per location")]
    fn insert_lineage_to_indexed_location_coherent(
        &mut self,
        reference: InMemoryLineageReference,
        indexed_location: IndexedLocation,
        _habitat: &AlmostInfiniteHabitat,
    ) {
        self.location_to_lineage_references
            .insert(indexed_location.location().clone(), reference);

        unsafe {
            self.lineages_store[Into::<usize>::into(reference)]
                .move_to_indexed_location(indexed_location)
        };
    }

    #[must_use]
    #[debug_requires(
        self[reference].indexed_location().unwrap().index() == 0,
        "only one lineage per location"
    )]
    fn extract_lineage_from_its_location_coherent(
        &mut self,
        reference: InMemoryLineageReference,
        _habitat: &AlmostInfiniteHabitat,
    ) -> IndexedLocation {
        let lineage: &Lineage = &self.lineages_store[Into::<usize>::into(reference)];

        let lineage_indexed_location = lineage.indexed_location().unwrap();

        let lineage_location = lineage_indexed_location.location();

        let lineage_reference_at_location = self
            .location_to_lineage_references
            .remove(lineage_location)
            .unwrap();

        unsafe {
            self.lineages_store[Into::<usize>::into(lineage_reference_at_location)]
                .remove_from_location()
        }
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

    fn emigrate(
        &mut self,
        _local_lineage_reference: InMemoryLineageReference,
    ) -> GlobalLineageReference {
        unimplemented!("TODO: Implement emigration for CoherentAlmostInfiniteLineageStore")
    }

    fn immigrate(
        &mut self,
        _habitat: &AlmostInfiniteHabitat,
        _global_reference: GlobalLineageReference,
        _indexed_location: IndexedLocation,
        _time_of_emigration: f64,
    ) -> InMemoryLineageReference {
        unimplemented!("TODO: Implement immigration for CoherentAlmostInfiniteLineageStore")
    }
}
