use core::marker::PhantomData;

use alloc::vec::Vec;

use necsim_core::cogs::{
    CoherentLineageStore, DispersalSampler, Habitat, LineageReference, RngCore,
};
use necsim_core::landscape::Location;

mod sampler;

#[allow(clippy::module_name_repetitions)]
#[derive(Debug)]
pub struct ClassicalActiveLineageSampler<
    H: Habitat,
    G: RngCore,
    D: DispersalSampler<H, G>,
    R: LineageReference<H>,
    S: CoherentLineageStore<H, R>,
> {
    active_lineage_references: Vec<R>,
    _marker: PhantomData<(H, G, D, S)>,
}

impl<
        H: Habitat,
        G: RngCore,
        D: DispersalSampler<H, G>,
        R: LineageReference<H>,
        S: CoherentLineageStore<H, R>,
    > ClassicalActiveLineageSampler<H, G, D, R, S>
{
    #[must_use]
    pub fn new(habitat: &H, lineage_store: &S) -> Self {
        let mut active_lineage_references = Vec::with_capacity(habitat.get_total_habitat());

        let landscape_extent = habitat.get_extent();

        for y in landscape_extent.y()..(landscape_extent.y() + landscape_extent.height()) {
            for x in landscape_extent.x()..(landscape_extent.x() + landscape_extent.width()) {
                active_lineage_references.extend_from_slice(
                    lineage_store.get_active_lineages_at_location(&Location::new(x, y)),
                );
            }
        }

        active_lineage_references.shrink_to_fit();

        Self {
            active_lineage_references,
            _marker: PhantomData::<(H, G, D, S)>,
        }
    }
}
