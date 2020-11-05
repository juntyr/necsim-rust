use necsim_core::cogs::{Habitat, IncoherentLineageStore, LineageStore};
use necsim_core::landscape::Location;
use necsim_core::lineage::Lineage;

use crate::cogs::lineage_reference::in_memory::InMemoryLineageReference;

use super::IncoherentInMemoryLineageStore;

#[contract_trait]
impl<H: Habitat> LineageStore<H, InMemoryLineageReference> for IncoherentInMemoryLineageStore<H> {
    #[must_use]
    fn new(sample_percentage: f64, habitat: &H) -> Self {
        Self::new_impl(sample_percentage, habitat)
    }

    #[must_use]
    fn get_number_total_lineages(&self) -> usize {
        self.lineages_store.len()
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
        self.landscape_extent.contains(&location),
        "location is inside landscape extent"
    )]
    fn insert_lineage_to_location_at_index(
        &mut self,
        reference: InMemoryLineageReference,
        location: Location,
        index_at_location: usize,
    ) {
        unsafe {
            self.lineages_store[Into::<usize>::into(reference)]
                .move_to_location(location, index_at_location)
        }
    }

    #[must_use]
    #[debug_requires(
        self.landscape_extent.contains(self[reference].location().unwrap()),
        "lineage's location is inside landscape extent"
    )]
    fn extract_lineage_from_its_location(
        &mut self,
        reference: InMemoryLineageReference,
    ) -> Location {
        unsafe { self.lineages_store[Into::<usize>::into(reference)].remove_from_location() }
    }
}
