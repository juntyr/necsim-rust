#![allow(clippy::type_complexity)]

use core::{marker::PhantomData, num::Wrapping};

use crate::cogs::{
    ActiveLineageSampler, CoalescenceSampler, DispersalSampler, EmigrationExit, EventSampler,
    Habitat, ImmigrationEntry, LineageReference, LineageStore, MathsCore, RngCore,
    SpeciationProbability, TurnoverRate,
};

#[derive(Debug)]
#[allow(clippy::module_name_repetitions)]
pub struct SimulationBuilder<
    M: MathsCore,
    H: Habitat<M>,
    G: RngCore<M>,
    R: LineageReference<M, H>,
    S: LineageStore<M, H, R>,
    X: EmigrationExit<M, H, G, R, S>,
    D: DispersalSampler<M, H, G>,
    C: CoalescenceSampler<M, H, R, S>,
    T: TurnoverRate<M, H>,
    N: SpeciationProbability<M, H>,
    E: EventSampler<M, H, G, R, S, X, D, C, T, N>,
    I: ImmigrationEntry<M>,
    A: ActiveLineageSampler<M, H, G, R, S, X, D, C, T, N, E, I>,
> {
    pub maths: PhantomData<M>,
    pub habitat: H,
    pub lineage_reference: PhantomData<R>,
    pub lineage_store: S,
    pub dispersal_sampler: D,
    pub coalescence_sampler: C,
    pub turnover_rate: T,
    pub speciation_probability: N,
    pub emigration_exit: X,
    pub event_sampler: E,
    pub active_lineage_sampler: A,
    pub rng: G,
    pub immigration_entry: I,
}

impl<
        M: MathsCore,
        H: Habitat<M>,
        G: RngCore<M>,
        R: LineageReference<M, H>,
        S: LineageStore<M, H, R>,
        X: EmigrationExit<M, H, G, R, S>,
        D: DispersalSampler<M, H, G>,
        C: CoalescenceSampler<M, H, R, S>,
        T: TurnoverRate<M, H>,
        N: SpeciationProbability<M, H>,
        E: EventSampler<M, H, G, R, S, X, D, C, T, N>,
        I: ImmigrationEntry<M>,
        A: ActiveLineageSampler<M, H, G, R, S, X, D, C, T, N, E, I>,
    > SimulationBuilder<M, H, G, R, S, X, D, C, T, N, E, I, A>
{
    #[allow(clippy::type_complexity)]
    pub fn build(self) -> Simulation<M, H, G, R, S, X, D, C, T, N, E, I, A> {
        let SimulationBuilder {
            maths,
            habitat,
            lineage_reference,
            lineage_store,
            dispersal_sampler,
            coalescence_sampler,
            turnover_rate,
            speciation_probability,
            emigration_exit,
            event_sampler,
            active_lineage_sampler,
            rng,
            immigration_entry,
        } = self;

        Simulation {
            maths,
            habitat,
            lineage_reference,
            lineage_store,
            dispersal_sampler,
            coalescence_sampler,
            turnover_rate,
            speciation_probability,
            emigration_exit,
            event_sampler,
            active_lineage_sampler,
            rng,
            immigration_entry,
            migration_balance: Wrapping(0_u64),
        }
    }
}

#[derive(Debug, TypeLayout)]
#[cfg_attr(feature = "cuda", derive(rust_cuda::common::LendRustToCuda))]
#[cfg_attr(feature = "cuda", r2cBound(H: rust_cuda::common::RustToCuda))]
#[cfg_attr(feature = "cuda", r2cBound(G: rust_cuda::common::RustToCuda))]
#[cfg_attr(feature = "cuda", r2cBound(D: rust_cuda::common::RustToCuda))]
#[cfg_attr(feature = "cuda", r2cBound(S: rust_cuda::common::RustToCuda))]
#[cfg_attr(feature = "cuda", r2cBound(X: rust_cuda::common::RustToCuda))]
#[cfg_attr(feature = "cuda", r2cBound(C: rust_cuda::common::RustToCuda))]
#[cfg_attr(feature = "cuda", r2cBound(T: rust_cuda::common::RustToCuda))]
#[cfg_attr(feature = "cuda", r2cBound(N: rust_cuda::common::RustToCuda))]
#[cfg_attr(feature = "cuda", r2cBound(E: rust_cuda::common::RustToCuda))]
#[cfg_attr(feature = "cuda", r2cBound(I: rust_cuda::common::RustToCuda))]
#[cfg_attr(feature = "cuda", r2cBound(A: rust_cuda::common::RustToCuda))]
#[repr(C)]
pub struct Simulation<
    M: MathsCore,
    H: Habitat<M>,
    G: RngCore<M>,
    R: LineageReference<M, H>,
    S: LineageStore<M, H, R>,
    X: EmigrationExit<M, H, G, R, S>,
    D: DispersalSampler<M, H, G>,
    C: CoalescenceSampler<M, H, R, S>,
    T: TurnoverRate<M, H>,
    N: SpeciationProbability<M, H>,
    E: EventSampler<M, H, G, R, S, X, D, C, T, N>,
    I: ImmigrationEntry<M>,
    A: ActiveLineageSampler<M, H, G, R, S, X, D, C, T, N, E, I>,
> {
    pub(super) maths: PhantomData<M>,
    #[cfg_attr(feature = "cuda", r2cEmbed)]
    pub(super) habitat: H,
    pub(super) lineage_reference: PhantomData<R>,
    #[cfg_attr(feature = "cuda", r2cEmbed)]
    pub(super) lineage_store: S,
    #[cfg_attr(feature = "cuda", r2cEmbed)]
    pub(super) dispersal_sampler: D,
    #[cfg_attr(feature = "cuda", r2cEmbed)]
    pub(super) coalescence_sampler: C,
    #[cfg_attr(feature = "cuda", r2cEmbed)]
    pub(super) turnover_rate: T,
    #[cfg_attr(feature = "cuda", r2cEmbed)]
    pub(super) speciation_probability: N,
    #[cfg_attr(feature = "cuda", r2cEmbed)]
    pub(super) emigration_exit: X,
    #[cfg_attr(feature = "cuda", r2cEmbed)]
    pub(super) event_sampler: E,
    #[cfg_attr(feature = "cuda", r2cEmbed)]
    pub(super) active_lineage_sampler: A,
    #[cfg_attr(feature = "cuda", r2cEmbed)]
    pub(super) rng: G,
    #[cfg_attr(feature = "cuda", r2cEmbed)]
    pub(super) immigration_entry: I,
    #[cfg_attr(feature = "cuda", r2cIgnore)]
    pub(super) migration_balance: Wrapping<u64>,
}

impl<
        M: MathsCore,
        H: Habitat<M>,
        G: RngCore<M>,
        R: LineageReference<M, H>,
        S: LineageStore<M, H, R>,
        X: EmigrationExit<M, H, G, R, S>,
        D: DispersalSampler<M, H, G>,
        C: CoalescenceSampler<M, H, R, S>,
        T: TurnoverRate<M, H>,
        N: SpeciationProbability<M, H>,
        E: EventSampler<M, H, G, R, S, X, D, C, T, N>,
        I: ImmigrationEntry<M>,
        A: ActiveLineageSampler<M, H, G, R, S, X, D, C, T, N, E, I>,
    > Simulation<M, H, G, R, S, X, D, C, T, N, E, I, A>
{
    #[inline]
    pub fn with_mut_split_active_lineage_sampler_and_rng<
        Q,
        F: FnOnce(
            &mut A,
            &mut super::partial::active_lineage_sampler::PartialSimulation<
                M,
                H,
                G,
                R,
                S,
                X,
                D,
                C,
                T,
                N,
                E,
            >,
            &mut G,
        ) -> Q,
    >(
        &mut self,
        func: F,
    ) -> Q {
        // Cast &self to a &PartialSimulation without the active lineage sampler
        //  and rng
        // This is only safe as PartialSimulation's type and layout is a prefix
        //  subsequence of Self's type and layout
        let partial_simulation = unsafe {
            &mut *(self as *mut Self)
                .cast::<super::partial::active_lineage_sampler::PartialSimulation<
                    M,
                    H,
                    G,
                    R,
                    S,
                    X,
                    D,
                    C,
                    T,
                    N,
                    E,
                >>()
        };

        func(
            &mut self.active_lineage_sampler,
            partial_simulation,
            &mut self.rng,
        )
    }

    #[inline]
    pub fn with_mut_split_event_sampler_and_rng<
        Q,
        F: FnOnce(
            &mut E,
            &super::partial::event_sampler::PartialSimulation<M, H, G, R, S, X, D, C, T, N>,
            &mut G,
        ) -> Q,
    >(
        &mut self,
        func: F,
    ) -> Q {
        // Cast &self to a &PartialSimulation without the event sampler and rng
        //  (the active lineage sampler is also removed implicitly)
        // This is only safe as PartialSimulation's type and layout is a prefix
        //  subsequence of Self's type and layout
        let partial_simulation = unsafe {
            &mut *(self as *mut Self).cast::<super::partial::event_sampler::PartialSimulation<
                M,
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

        func(&mut self.event_sampler, partial_simulation, &mut self.rng)
    }

    pub fn rng_mut(&mut self) -> &mut G {
        &mut self.rng
    }

    pub fn active_lineage_sampler(&self) -> &A {
        &self.active_lineage_sampler
    }

    pub fn active_lineage_sampler_mut(&mut self) -> &mut A {
        &mut self.active_lineage_sampler
    }

    pub fn lineage_store(&self) -> &S {
        &self.lineage_store
    }

    pub fn lineage_store_mut(&mut self) -> &mut S {
        &mut self.lineage_store
    }

    pub fn event_sampler(&self) -> &E {
        &self.event_sampler
    }

    pub fn event_sampler_mut(&mut self) -> &mut E {
        &mut self.event_sampler
    }

    pub fn speciation_probability(&self) -> &N {
        &self.speciation_probability
    }

    pub fn turnover_rate(&self) -> &T {
        &self.turnover_rate
    }

    pub fn habitat(&self) -> &H {
        &self.habitat
    }

    pub fn dispersal_sampler(&self) -> &D {
        &self.dispersal_sampler
    }

    pub fn coalescence_sampler(&self) -> &C {
        &self.coalescence_sampler
    }

    pub fn emigration_exit(&self) -> &X {
        &self.emigration_exit
    }

    pub fn emigration_exit_mut(&mut self) -> &mut X {
        &mut self.emigration_exit
    }

    pub fn immigration_entry_mut(&mut self) -> &mut I {
        &mut self.immigration_entry
    }
}
