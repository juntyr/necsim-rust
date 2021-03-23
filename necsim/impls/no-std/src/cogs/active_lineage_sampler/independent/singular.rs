use necsim_core::cogs::{
    DispersalSampler, EmigrationExit, Habitat, PrimeableRng, SingularActiveLineageSampler,
    SpeciationProbability, TurnoverRate,
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
        X: EmigrationExit<H, G, GlobalLineageReference, IndependentLineageStore<H>>,
        D: DispersalSampler<H, G>,
        T: TurnoverRate<H>,
        N: SpeciationProbability<H>,
        J: EventTimeSampler<H, G, T>,
    >
    SingularActiveLineageSampler<
        H,
        G,
        GlobalLineageReference,
        IndependentLineageStore<H>,
        X,
        D,
        IndependentCoalescenceSampler<H>,
        T,
        N,
        IndependentEventSampler<H, G, X, D, T, N>,
        NeverImmigrationEntry,
    > for IndependentActiveLineageSampler<H, G, X, D, T, N, J>
{
    #[must_use]
    #[inline]
    fn replace_active_lineage(&mut self, active_lineage: Option<Lineage>) -> Option<Lineage> {
        // `core::mem::replace()` would be semantically better
        //  - but `clone()` does not spill to local memory
        let old_active_lineage = self.active_lineage.clone();

        self.active_lineage = active_lineage;

        old_active_lineage
    }
}
