use core::marker::PhantomData;

use crate::cogs::{
    CoalescenceSampler, DispersalSampler, EmigrationExit, EventSampler, Habitat, LineageReference,
    LineageStore, RngCore, SpeciationProbability, TurnoverRate,
};

#[repr(C)]
pub struct PartialSimulation<
    H: Habitat,
    G: RngCore,
    R: LineageReference<H>,
    S: LineageStore<H, R>,
    X: EmigrationExit<H, G, R, S>,
    D: DispersalSampler<H, G>,
    C: CoalescenceSampler<H, R, S>,
    T: TurnoverRate<H>,
    N: SpeciationProbability<H>,
    E: EventSampler<H, G, R, S, X, D, C, T, N>,
> {
    pub habitat: H,
    pub lineage_reference: PhantomData<R>,
    pub lineage_store: S,
    pub emigration_exit: X,
    pub dispersal_sampler: D,
    pub coalescence_sampler: C,
    pub turnover_rate: T,
    pub speciation_probability: N,
    pub event_sampler: E,
    // priv
    _rng: PhantomData<G>,
}

impl<
        H: Habitat,
        G: RngCore,
        R: LineageReference<H>,
        S: LineageStore<H, R>,
        X: EmigrationExit<H, G, R, S>,
        D: DispersalSampler<H, G>,
        C: CoalescenceSampler<H, R, S>,
        T: TurnoverRate<H>,
        N: SpeciationProbability<H>,
        E: EventSampler<H, G, R, S, X, D, C, T, N>,
    > PartialSimulation<H, G, R, S, X, D, C, T, N, E>
{
    #[inline]
    pub fn with_split_event_sampler<
        Q,
        F: FnOnce(&E, &super::event_sampler::PartialSimulation<H, G, R, S, X, D, C, T, N>) -> Q,
    >(
        &self,
        func: F,
    ) -> Q {
        // Cast &self to a &PartialSimulation without the event sampler
        // This is only safe as PartialSimulation's type and layout is a prefix
        //  subsequence of Self's type and layout
        let partial_simulation = unsafe {
            &*(self as *const Self).cast::<super::event_sampler::PartialSimulation<
                H,
                G,
                R,
                S,
                X,
                D,
                C,
                T,
                N,
            >>()
        };

        func(&self.event_sampler, partial_simulation)
    }

    #[inline]
    pub fn with_mut_split_event_sampler<
        Q,
        F: FnOnce(
            &mut E,
            &mut super::event_sampler::PartialSimulation<H, G, R, S, X, D, C, T, N>,
        ) -> Q,
    >(
        &mut self,
        func: F,
    ) -> Q {
        // Cast &mut self to a &mut PartialSimulation without the event sampler
        // This is only safe as PartialSimulation's type and layout is a prefix
        //  subsequence of Self's type and layout
        let partial_simulation = unsafe {
            &mut *(self as *mut Self).cast::<super::event_sampler::PartialSimulation<
                H,
                G,
                R,
                S,
                X,
                D,
                C,
                T,
                N,
            >>()
        };

        func(&mut self.event_sampler, partial_simulation)
    }
}
