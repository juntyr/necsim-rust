use necsim_core::cogs::{Habitat, LineageStore};
use necsim_core::landscape::Location;
use necsim_core::lineage::Lineage;

use necsim_impls_no_std::cogs::lineage_reference::in_memory::InMemoryLineageReference;
use necsim_impls_no_std::cogs::lineage_store::coherent::CoherentLineageStore;

use super::CoherentInMemoryLineageStore;

#[contract_trait]
impl<H: Habitat> LineageStore<H, InMemoryLineageReference> for CoherentInMemoryLineageStore<H> {
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
        &self.location_to_lineage_references[(
            (location.y() - self.landscape_extent.y()) as usize,
            (location.x() - self.landscape_extent.x()) as usize,
        )]
    }

    #[must_use]
    fn get(&self, reference: InMemoryLineageReference) -> Option<&Lineage> {
        self.lineages_store.get(Into::<usize>::into(reference))
    }

    #[debug_requires(
        self.landscape_extent.contains(&location),
        "location is inside landscape extent"
    )]
    fn add_lineage_to_location(&mut self, reference: InMemoryLineageReference, location: Location) {
        let lineages_at_location = &mut self.location_to_lineage_references[(
            (location.y() - self.landscape_extent.y()) as usize,
            (location.x() - self.landscape_extent.x()) as usize,
        )];

        lineages_at_location.push(reference);

        unsafe {
            self.lineages_store[Into::<usize>::into(reference)]
                .move_to_location(location, lineages_at_location.len() - 1)
        };
    }

    #[debug_requires(
        self.landscape_extent.contains(self[reference].location()),
        "lineage's location is inside landscape extent"
    )]
    fn remove_lineage_from_its_location(&mut self, reference: InMemoryLineageReference) {
        let lineage: &Lineage = &self.lineages_store[Into::<usize>::into(reference)];

        let lineages_at_location = &mut self.location_to_lineage_references[(
            (lineage.location().y() - self.landscape_extent.y()) as usize,
            (lineage.location().x() - self.landscape_extent.x()) as usize,
        )];

        if let Some(last_lineage_at_location) = lineages_at_location.pop() {
            let lineage_index_at_location = lineage.index_at_location();

            if lineage_index_at_location < lineages_at_location.len() {
                lineages_at_location[lineage_index_at_location] = last_lineage_at_location;

                unsafe {
                    self.lineages_store[Into::<usize>::into(last_lineage_at_location)]
                        .update_index_at_location(lineage_index_at_location)
                };
            }
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
}

#[contract_trait]
impl<H: Habitat> CoherentLineageStore<H, InMemoryLineageReference>
    for CoherentInMemoryLineageStore<H>
{
}
