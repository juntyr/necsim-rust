#![allow(clippy::used_underscore_binding)]
#![allow(clippy::empty_enum)]

use core::{marker::PhantomData, num::Wrapping};

use crate::cogs::{
    ActiveLineageSampler, CoalescenceSampler, DispersalSampler, EmigrationExit, EventSampler,
    Habitat, ImmigrationEntry, LineageReference, LineageStore, RngCore, SpeciationProbability,
    TurnoverRate,
};

#[derive(TypedBuilder, Debug)]
#[cfg_attr(feature = "cuda", derive(RustToCuda, LendToCuda))]
#[cfg_attr(feature = "cuda", r2cBound(H: rust_cuda::common::RustToCuda))]
#[cfg_attr(feature = "cuda", r2cBound(G: rust_cuda::common::RustToCuda))]
#[cfg_attr(feature = "cuda", r2cBound(D: rust_cuda::common::RustToCuda))]
#[cfg_attr(feature = "cuda", r2cBound(R: rustacuda_core::DeviceCopy))]
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
    I: ImmigrationEntry,
    A: ActiveLineageSampler<H, G, R, S, X, D, C, T, N, E, I>,
> {
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
    #[builder(default = Wrapping(0), setter(skip))]
    pub(super) migration_balance: Wrapping<u64>,
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
        I: ImmigrationEntry,
        A: ActiveLineageSampler<H, G, R, S, X, D, C, T, N, E, I>,
    > Simulation<H, G, R, S, X, D, C, T, N, E, I, A>
{
    #[inline]
    pub fn with_mut_split_active_lineage_sampler_and_rng<
        Q,
        F: FnOnce(
            &mut A,
            &mut super::partial::active_lineager_sampler::PartialSimulation<
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
                .cast::<super::partial::active_lineager_sampler::PartialSimulation<
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
            &super::partial::event_sampler::PartialSimulation<H, G, R, S, X, D, C, T, N>,
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
