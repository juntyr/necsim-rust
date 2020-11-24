use necsim_core::cogs::{
    DispersalSampler, HabitatToU64Injection, IncoherentLineageStore, LineageReference,
    PrimeableRng, SingularActiveLineageSampler,
};

use crate::cogs::{
    coalescence_sampler::independent::IndependentCoalescenceSampler,
    event_sampler::independent::IndependentEventSampler,
};

use super::IndependentActiveLineageSampler;

impl<
        H: HabitatToU64Injection,
        G: PrimeableRng<H>,
        D: DispersalSampler<H, G>,
        R: LineageReference<H>,
        S: IncoherentLineageStore<H, R>,
    >
    SingularActiveLineageSampler<
        H,
        G,
        D,
        R,
        S,
        IndependentCoalescenceSampler<H, G, R, S>,
        IndependentEventSampler<H, G, D, R, S>,
    > for IndependentActiveLineageSampler<H, G, D, R, S>
{
    #[must_use]
    fn replace_active_lineage(&mut self, active_lineage_reference: Option<R>) -> Option<R> {
        core::mem::replace(&mut self.active_lineage_reference, active_lineage_reference)
    }
}
