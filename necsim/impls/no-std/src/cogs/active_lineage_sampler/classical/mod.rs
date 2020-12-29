use core::marker::PhantomData;

use alloc::vec::Vec;

use necsim_core::cogs::{
    CoherentLineageStore, DispersalSampler, Habitat, LineageReference, RngCore,
    SpeciationProbability,
};

mod sampler;

#[allow(clippy::module_name_repetitions)]
#[derive(Debug)]
pub struct ClassicalActiveLineageSampler<
    H: Habitat,
    G: RngCore,
    N: SpeciationProbability<H>,
    D: DispersalSampler<H, G>,
    R: LineageReference<H>,
    S: CoherentLineageStore<H, R>,
> {
    active_lineage_references: Vec<R>,
    last_event_time: f64,
    _marker: PhantomData<(H, G, N, D, S)>,
}

impl<
        H: Habitat,
        G: RngCore,
        N: SpeciationProbability<H>,
        D: DispersalSampler<H, G>,
        R: LineageReference<H>,
        S: CoherentLineageStore<H, R>,
    > ClassicalActiveLineageSampler<H, G, N, D, R, S>
{
    #[must_use]
    pub fn new(lineage_store: &S) -> Self {
        let mut active_lineage_references =
            Vec::with_capacity(lineage_store.get_number_total_lineages());

        lineage_store.iter_active_locations().for_each(|location| {
            active_lineage_references
                .extend_from_slice(lineage_store.get_active_lineages_at_location(&location))
        });

        active_lineage_references.shrink_to_fit();

        Self {
            active_lineage_references,
            last_event_time: 0.0_f64,
            _marker: PhantomData::<(H, G, N, D, S)>,
        }
    }
}
