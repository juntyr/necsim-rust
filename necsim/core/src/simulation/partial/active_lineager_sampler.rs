use core::marker::PhantomData;

use crate::cogs::{
    CoalescenceSampler, DispersalSampler, EmigrationExit, EventSampler, Habitat, LineageReference,
    LineageStore, RngCore, SpeciationProbability,
};

#[repr(C)]
pub struct PartialSimulation<
    H: Habitat,
    G: RngCore,
    N: SpeciationProbability<H>,
    D: DispersalSampler<H, G>,
    R: LineageReference<H>,
    S: LineageStore<H, R>,
    X: EmigrationExit<H, G, N, D, R, S>,
    C: CoalescenceSampler<H, G, R, S>,
    E: EventSampler<H, G, N, D, R, S, X, C>,
> {
    pub habitat: H,
    pub speciation_probability: N,
    pub dispersal_sampler: D,
    pub lineage_reference: PhantomData<R>,
    pub lineage_store: S,
    pub emigration_exit: X,
    pub coalescence_sampler: C,
    pub event_sampler: E,
    pub rng: PhantomData<G>,
}

impl<
        H: Habitat,
        G: RngCore,
        N: SpeciationProbability<H>,
        D: DispersalSampler<H, G>,
        R: LineageReference<H>,
        S: LineageStore<H, R>,
        X: EmigrationExit<H, G, N, D, R, S>,
        C: CoalescenceSampler<H, G, R, S>,
        E: EventSampler<H, G, N, D, R, S, X, C>,
    > PartialSimulation<H, G, N, D, R, S, X, C, E>
{
    #[inline]
    pub fn with_split_event_sampler<
        Q,
        F: FnOnce(&E, &super::event_sampler::PartialSimulation<H, G, N, D, R, S, X, C>) -> Q,
    >(
        &self,
        func: F,
    ) -> Q {
        // Cast &self to a &PartialSimulation without the event sampler
        // This is only safe as both types have the same fields and layout except for
        // event_sampler in Self at the end
        let partial_simulation = unsafe {
            &*(self as *const Self
                as *const super::event_sampler::PartialSimulation<H, G, N, D, R, S, X, C>)
        };

        func(&self.event_sampler, partial_simulation)
    }

    #[inline]
    pub fn with_mut_split_event_sampler<
        Q,
        F: FnOnce(&mut E, &mut super::event_sampler::PartialSimulation<H, G, N, D, R, S, X, C>) -> Q,
    >(
        &mut self,
        func: F,
    ) -> Q {
        // Cast &mut self to a &mut PartialSimulation without the event sampler
        // This is only safe as both types have the same fields and layout except for
        // event_sampler in Self at the end
        #[allow(clippy::cast_ref_to_mut)]
        let partial_simulation = unsafe {
            &mut *(self as *const Self
                as *mut super::event_sampler::PartialSimulation<H, G, N, D, R, S, X, C>)
        };

        func(&mut self.event_sampler, partial_simulation)
    }
}
