use necsim_core::cogs::{
    DispersalSampler, EmigrationExit, F64Core, Habitat, PrimeableRng, SpeciationProbability,
    TurnoverRate,
};

use necsim_core::lineage::{GlobalLineageReference, Lineage};

use crate::cogs::{
    active_lineage_sampler::singular::SingularActiveLineageSampler,
    coalescence_sampler::independent::IndependentCoalescenceSampler,
    event_sampler::independent::IndependentEventSampler,
    immigration_entry::never::NeverImmigrationEntry,
    lineage_store::independent::IndependentLineageStore,
};

use super::{EventTimeSampler, IndependentActiveLineageSampler};

impl<
        F: F64Core,
        H: Habitat<F>,
        G: PrimeableRng<F>,
        X: EmigrationExit<F, H, G, GlobalLineageReference, IndependentLineageStore<F, H>>,
        D: DispersalSampler<F, H, G>,
        T: TurnoverRate<F, H>,
        N: SpeciationProbability<F, H>,
        J: EventTimeSampler<F, H, G, T>,
    >
    SingularActiveLineageSampler<
        F,
        H,
        G,
        GlobalLineageReference,
        IndependentLineageStore<F, H>,
        X,
        D,
        IndependentCoalescenceSampler<F, H>,
        T,
        N,
        IndependentEventSampler<F, H, G, X, D, T, N>,
        NeverImmigrationEntry,
    > for IndependentActiveLineageSampler<F, H, G, X, D, T, N, J>
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
