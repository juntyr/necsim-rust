use core::marker::PhantomData;

use crate::cogs::{
    CoalescenceSampler, DispersalSampler, EmigrationExit, EventSampler, Habitat, LineageStore,
    MathsCore, Rng, SpeciationProbability, TurnoverRate,
};

#[repr(C)]
pub struct PartialSimulation<
    M: MathsCore,
    H: Habitat<M>,
    G: Rng<M>,
    S: LineageStore<M, H>,
    X: EmigrationExit<M, H, G, S>,
    D: DispersalSampler<M, H, G>,
    C: CoalescenceSampler<M, H, S>,
    T: TurnoverRate<M, H>,
    N: SpeciationProbability<M, H>,
    E: EventSampler<M, H, G, S, X, D, C, T, N>,
> {
    pub maths: PhantomData<M>,
    pub habitat: H,
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
        M: MathsCore,
        H: Habitat<M>,
        G: Rng<M>,
        S: LineageStore<M, H>,
        X: EmigrationExit<M, H, G, S>,
        D: DispersalSampler<M, H, G>,
        C: CoalescenceSampler<M, H, S>,
        T: TurnoverRate<M, H>,
        N: SpeciationProbability<M, H>,
        E: EventSampler<M, H, G, S, X, D, C, T, N>,
    > PartialSimulation<M, H, G, S, X, D, C, T, N, E>
{
    #[inline]
    pub fn with_split_event_sampler<
        Q,
        F: FnOnce(&E, &super::event_sampler::PartialSimulation<M, H, G, S, X, D, C, T, N>) -> Q,
    >(
        &self,
        func: F,
    ) -> Q {
        // Cast &self to a &PartialSimulation without the event sampler
        // This is only safe as PartialSimulation's type and layout is a prefix
        //  subsequence of Self's type and layout
        let partial_simulation = unsafe {
            &*(self as *const Self).cast::<super::event_sampler::PartialSimulation<
                M,
                H,
                G,
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
            &mut super::event_sampler::PartialSimulation<M, H, G, S, X, D, C, T, N>,
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
                M,
                H,
                G,
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
