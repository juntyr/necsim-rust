use necsim_core::cogs::{
    DispersalSampler, EmigrationExit, Habitat, MathsCore, PrimeableRng, SpeciationProbability,
    TurnoverRate,
};

use necsim_core::lineage::{GlobalLineageReference, Lineage};
use necsim_core_bond::NonNegativeF64;

use crate::cogs::{
    active_lineage_sampler::singular::SingularActiveLineageSampler,
    coalescence_sampler::independent::IndependentCoalescenceSampler,
    event_sampler::independent::IndependentEventSampler,
    immigration_entry::never::NeverImmigrationEntry,
    lineage_store::independent::IndependentLineageStore,
};

use super::{EventTimeSampler, IndependentActiveLineageSampler};

impl<
        M: MathsCore,
        H: Habitat<M>,
        G: PrimeableRng<M>,
        X: EmigrationExit<M, H, G, GlobalLineageReference, IndependentLineageStore<M, H>>,
        D: DispersalSampler<M, H, G>,
        T: TurnoverRate<M, H>,
        N: SpeciationProbability<M, H>,
        J: EventTimeSampler<M, H, G, T>,
    >
    SingularActiveLineageSampler<
        M,
        H,
        G,
        GlobalLineageReference,
        IndependentLineageStore<M, H>,
        X,
        D,
        IndependentCoalescenceSampler<M, H>,
        T,
        N,
        IndependentEventSampler<M, H, G, X, D, T, N>,
        NeverImmigrationEntry,
    > for IndependentActiveLineageSampler<M, H, G, X, D, T, N, J>
{
    #[must_use]
    #[inline]
    fn replace_active_lineage(&mut self, active_lineage: Option<Lineage>) -> Option<Lineage> {
        // `core::mem::replace()` would be semantically better
        //  - but `clone()` does not spill to local memory
        let old_active_lineage = self.active_lineage.clone();

        self.active_lineage = active_lineage;

        self.last_event_time = match &self.active_lineage {
            None => NonNegativeF64::zero(),
            Some(lineage) => lineage.last_event_time,
        };

        old_active_lineage
    }
}
