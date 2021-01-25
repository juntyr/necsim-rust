use necsim_core::cogs::{
    DispersalSampler, EmigrationExit, Habitat, PrimeableRng, SingularActiveLineageSampler,
    SpeciationProbability,
};

use necsim_core::lineage::{GlobalLineageReference, Lineage};

use crate::cogs::{
    coalescence_sampler::independent::IndependentCoalescenceSampler,
    event_sampler::independent::IndependentEventSampler,
    immigration_entry::never::NeverImmigrationEntry,
    lineage_store::independent::IndependentLineageStore,
};

use super::{EventTimeSampler, IndependentActiveLineageSampler};

impl<
        H: Habitat,
        G: PrimeableRng<H>,
        N: SpeciationProbability<H>,
        T: EventTimeSampler<H, G>,
        D: DispersalSampler<H, G>,
        X: EmigrationExit<H, G, N, D, GlobalLineageReference, IndependentLineageStore<H>>,
    >
    SingularActiveLineageSampler<
        H,
        G,
        N,
        D,
        GlobalLineageReference,
        IndependentLineageStore<H>,
        X,
        IndependentCoalescenceSampler<H>,
        IndependentEventSampler<H, G, N, D, X>,
        NeverImmigrationEntry,
    > for IndependentActiveLineageSampler<H, G, N, T, D, X>
{
    #[must_use]
    #[inline]
    fn replace_active_lineage(&mut self, active_lineage: Option<Lineage>) -> Option<Lineage> {
        #[allow(clippy::option_if_let_else)]
        if let Some(active_lineage) = active_lineage {
            self.active_lineage.replace(active_lineage)
        } else {
            self.active_lineage.take()
        }
    }
}
