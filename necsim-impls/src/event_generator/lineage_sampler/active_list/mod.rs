use std::ops::Index;

use necsim_core::landscape::{Landscape, Location};
use necsim_core::lineage::Lineage;
use necsim_core::simulation::SimulationSettings;

use crate::event_generator::lineage_sampler::global_store::{GlobalLineageStore, LineageReference};

mod coalescence;
mod sampler;

pub struct ActiveLineageListSampler {
    lineages_store: GlobalLineageStore,
    active_lineage_references: Vec<LineageReference>,
}

impl ActiveLineageListSampler {
    #[must_use]
    pub fn new(settings: &SimulationSettings<impl Landscape>) -> Self {
        let lineages_store = GlobalLineageStore::new(settings);

        let mut active_lineage_references =
            Vec::with_capacity(settings.landscape().get_total_habitat());

        let landscape_extent = settings.landscape().get_extent();

        for y in landscape_extent.y()..(landscape_extent.y() + landscape_extent.height()) {
            for x in landscape_extent.x()..(landscape_extent.x() + landscape_extent.width()) {
                active_lineage_references.extend_from_slice(
                    lineages_store.get_active_lineages_at_location(&Location::new(x, y)),
                );
            }
        }

        active_lineage_references.shrink_to_fit();

        Self {
            lineages_store,
            active_lineage_references,
        }
    }
}

impl Index<LineageReference> for ActiveLineageListSampler {
    type Output = Lineage;

    #[must_use]
    fn index(&self, reference: LineageReference) -> &Self::Output {
        &self.lineages_store[reference]
    }
}
