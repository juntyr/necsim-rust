use core::marker::PhantomData;

use crate::cogs::{
    CoalescenceSampler, DispersalSampler, EventSampler, Habitat, LineageReference, LineageStore,
    RngCore,
};

#[repr(C)]
pub struct PartialSimulation<
    H: Habitat,
    G: RngCore,
    D: DispersalSampler<H, G>,
    R: LineageReference<H>,
    S: LineageStore<H, R>,
    C: CoalescenceSampler<H, G, R, S>,
    E: EventSampler<H, G, D, R, S, C>,
> {
    pub speciation_probability_per_generation: f64,
    pub habitat: H,
    pub dispersal_sampler: D,
    pub lineage_reference: PhantomData<R>,
    pub lineage_store: S,
    pub coalescence_sampler: C,
    pub event_sampler: E,
    pub rng: PhantomData<G>,
}

impl<
        H: Habitat,
        G: RngCore,
        D: DispersalSampler<H, G>,
        R: LineageReference<H>,
        S: LineageStore<H, R>,
        C: CoalescenceSampler<H, G, R, S>,
        E: EventSampler<H, G, D, R, S, C>,
    > PartialSimulation<H, G, D, R, S, C, E>
{
    pub fn with_split_event_sampler<
        Q,
        F: FnOnce(&E, &super::event_sampler::PartialSimulation<H, G, D, R, S, C>) -> Q,
    >(
        &self,
        func: F,
    ) -> Q {
        // Cast &self to a &PartialSimulation without the event sampler
        // This is only safe as both types have the same fields and layout except for
        // event_sampler in Self at the end
        let partial_simulation = unsafe {
            &*(self as *const Self
                as *const super::event_sampler::PartialSimulation<H, G, D, R, S, C>)
        };

        func(&self.event_sampler, partial_simulation)
    }
}
