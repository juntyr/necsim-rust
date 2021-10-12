use core::marker::PhantomData;

use crate::cogs::{
    CoalescenceSampler, DispersalSampler, EmigrationExit, EventSampler, Habitat, LineageReference,
    LineageStore, RngCore, SpeciationProbability, TurnoverRate, F64Core
};

#[repr(C)]
pub struct PartialSimulation<
    F: F64Core,
    H: Habitat<F>,
    G: RngCore<F>,
    R: LineageReference<F,H>,
    S: LineageStore<F, H, R>,
    X: EmigrationExit<F, H, G, R, S>,
    D: DispersalSampler<F, H, G>,
    C: CoalescenceSampler<F, H, R, S>,
    T: TurnoverRate<F, H>,
    N: SpeciationProbability<F, H>,
    E: EventSampler<F, H, G, R, S, X, D, C, T, N>,
> {
    pub f64_core: PhantomData<F>,
    pub habitat: H,
    pub lineage_reference: PhantomData<R>,
    pub lineage_store: S,
    pub dispersal_sampler: D,
    pub coalescence_sampler: C,
    pub turnover_rate: T,
    pub speciation_probability: N,
    pub emigration_exit: X,
    pub event_sampler: E,
    // priv
    _rng: PhantomData<G>,
}

impl<
        F: F64Core,
        H: Habitat<F>,
        G: RngCore<F>,
        R: LineageReference<F, H>,
        S: LineageStore<F, H, R>,
        X: EmigrationExit<F, H, G, R, S>,
        D: DispersalSampler<F, H, G>,
        C: CoalescenceSampler<F, H, R, S>,
        T: TurnoverRate<F, H>,
        N: SpeciationProbability<F, H>,
        E: EventSampler<F, H, G, R, S, X, D, C, T, N>,
    > PartialSimulation<F, H, G, R, S, X, D, C, T, N, E>
{
    #[inline]
    pub fn with_split_event_sampler<
        Q,
        W: FnOnce(&E, &super::event_sampler::PartialSimulation<F, H, G, R, S, X, D, C, T, N>) -> Q,
    >(
        &self,
        func: W,
    ) -> Q {
        // Cast &self to a &PartialSimulation without the event sampler
        // This is only safe as PartialSimulation's type and layout is a prefix
        //  subsequence of Self's type and layout
        let partial_simulation = unsafe {
            &*(self as *const Self).cast::<super::event_sampler::PartialSimulation<
                F,
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
        W: FnOnce(
            &mut E,
            &mut super::event_sampler::PartialSimulation<F, H, G, R, S, X, D, C, T, N>,
        ) -> Q,
    >(
        &mut self,
        func: W,
    ) -> Q {
        // Cast &mut self to a &mut PartialSimulation without the event sampler
        // This is only safe as PartialSimulation's type and layout is a prefix
        //  subsequence of Self's type and layout
        let partial_simulation = unsafe {
            &mut *(self as *mut Self).cast::<super::event_sampler::PartialSimulation<
                F,
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
