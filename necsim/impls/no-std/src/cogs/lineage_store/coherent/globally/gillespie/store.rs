use necsim_core::{
    cogs::{GloballyCoherentLineageStore, Habitat, LineageStore, LocallyCoherentLineageStore},
    landscape::{IndexedLocation, Location, LocationIterator},
    lineage::{GlobalLineageReference, Lineage},
};

use crate::cogs::lineage_reference::in_memory::InMemoryLineageReference;

use super::GillespieLineageStore;

#[contract_trait]
impl<H: Habitat> LineageStore<H, InMemoryLineageReference> for GillespieLineageStore<H> {
    #[allow(clippy::type_complexity)]
    type LineageReferenceIterator<'a> = core::iter::Map<
        slab::Iter<'a, Lineage>,
        fn((usize, &'a Lineage)) -> InMemoryLineageReference,
    >;

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
        self.lineages_store.get(Into::<usize>::into(reference))
    }
}

#[contract_trait]
impl<H: Habitat> LocallyCoherentLineageStore<H, InMemoryLineageReference>
    for GillespieLineageStore<H>
{
    #[must_use]
    fn get_active_global_lineage_reference_at_indexed_location(
        &self,
        indexed_location: &IndexedLocation,
        _habitat: &H,
    ) -> Option<&GlobalLineageReference> {
        self.indexed_location_to_lineage_reference
            .get(indexed_location)
            .map(|(global_reference, _index)| global_reference)
    }

    fn insert_lineage_to_indexed_location_locally_coherent(
        &mut self,
        reference: InMemoryLineageReference,
        indexed_location: IndexedLocation,
        habitat: &H,
    ) {
        let lineage: &Lineage = &self.lineages_store[Into::<usize>::into(reference)];

        let lineages_at_location = &mut self.location_to_lineage_references[(
            (indexed_location.location().y() - habitat.get_extent().y()) as usize,
            (indexed_location.location().x() - habitat.get_extent().x()) as usize,
        )];

        self.indexed_location_to_lineage_reference.insert(
            indexed_location.clone(),
            (
                lineage.global_reference().clone(),
                lineages_at_location.len(),
            ),
        );
        lineages_at_location.push(reference);

        unsafe {
            self.lineages_store[Into::<usize>::into(reference)]
                .move_to_indexed_location(indexed_location)
        };
    }

    #[must_use]
    fn extract_lineage_from_its_location_locally_coherent(
        &mut self,
        reference: InMemoryLineageReference,
        event_time: f64,
        habitat: &H,
    ) -> IndexedLocation {
        let lineage_indexed_location = unsafe {
            self.lineages_store[Into::<usize>::into(reference)].remove_from_location(event_time)
        };

        // We know from the trait preconditions that this value exists
        let (_global_reference, local_index) = self
            .indexed_location_to_lineage_reference
            .remove(&lineage_indexed_location)
            .unwrap();

        let lineages_at_location = &mut self.location_to_lineage_references[(
            (lineage_indexed_location.location().y() - habitat.get_extent().y()) as usize,
            (lineage_indexed_location.location().x() - habitat.get_extent().x()) as usize,
        )];

        lineages_at_location.swap_remove(local_index);

        if let Some(replacement_local_reference) = lineages_at_location.get(local_index) {
            // Only executed when the reference was not the last item in the unordered index
            if let Some(replacement_location) = self.lineages_store
                [Into::<usize>::into(*replacement_local_reference)]
            .indexed_location()
            {
                // Always executed as the replacement lineage is active
                if let Some((_replacement_global_reference, replacement_index)) = self
                    .indexed_location_to_lineage_reference
                    .get_mut(replacement_location)
                {
                    // Always executed as the replacement lineage is active
                    *replacement_index = local_index;
                }
            }
        }

        lineage_indexed_location
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
        habitat: &H,
        global_reference: GlobalLineageReference,
        indexed_location: IndexedLocation,
        time_of_emigration: f64,
    ) -> InMemoryLineageReference {
        let lineage = Lineage::immigrate(
            global_reference,
            indexed_location.clone(),
            time_of_emigration,
        );

        let lineages_at_location = &mut self.location_to_lineage_references[(
            (indexed_location.location().y() - habitat.get_extent().y()) as usize,
            (indexed_location.location().x() - habitat.get_extent().x()) as usize,
        )];

        self.indexed_location_to_lineage_reference.insert(
            indexed_location,
            (
                lineage.global_reference().clone(),
                lineages_at_location.len(),
            ),
        );

        let local_lineage_reference =
            InMemoryLineageReference::from(self.lineages_store.insert(lineage));

        lineages_at_location.push(local_lineage_reference);

        local_lineage_reference
    }
}

#[contract_trait]
impl<H: Habitat> GloballyCoherentLineageStore<H, InMemoryLineageReference>
    for GillespieLineageStore<H>
{
    type LocationIterator<'a> = LocationIterator;

    #[must_use]
    fn iter_active_locations(&self, habitat: &H) -> Self::LocationIterator<'_> {
        habitat.get_extent().iter()
    }

    #[must_use]
    fn get_active_local_lineage_references_at_location_unordered(
        &self,
        location: &Location,
        habitat: &H,
    ) -> &[InMemoryLineageReference] {
        &self.location_to_lineage_references[(
            (location.y() - habitat.get_extent().y()) as usize,
            (location.x() - habitat.get_extent().x()) as usize,
        )]
    }
}
