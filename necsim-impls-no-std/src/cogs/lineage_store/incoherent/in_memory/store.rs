use necsim_core::cogs::{Habitat, LineageStore};
use necsim_core::landscape::Location;
use necsim_core::lineage::Lineage;

use crate::cogs::lineage_reference::in_memory::InMemoryLineageReference;
use crate::cogs::lineage_store::incoherent::IncoherentLineageStore;

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
    #[debug_requires(
        self.landscape_extent.contains(location),
        "location is inside landscape extent"
    )]
    fn get_active_lineages_at_location(&self, location: &Location) -> &[InMemoryLineageReference] {
        unimplemented!("Need to return single element we are tracking")
    }

    #[must_use]
    fn get(&self, reference: InMemoryLineageReference) -> Option<&Lineage> {
        self.lineages_store.get(Into::<usize>::into(reference))
    }

    #[debug_requires(
        self.landscape_extent.contains(&location),
        "location is inside landscape extent"
    )]
    fn add_lineage_to_location(
        &mut self,
        _reference: InMemoryLineageReference,
        location: Location,
    ) {
        unimplemented!("Need to also pass index at location")
        /*unsafe {
            self.lineages_store[Into::<usize>::into(reference)]
                .move_to_location(location, lineages_at_location.len() - 1)
        };*/
    }

    #[debug_requires(
        self.landscape_extent.contains(self[reference].location()),
        "lineage's location is inside landscape extent"
    )]
    fn remove_lineage_from_its_location(&mut self, reference: InMemoryLineageReference) {
        unimplemented!("How can we remove from a location when we do not store if it has been removed right now -> maybe change index at location to optional?")
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
}
