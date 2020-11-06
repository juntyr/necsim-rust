#![allow(clippy::used_underscore_binding)]
#![allow(clippy::empty_enum)]

use core::marker::PhantomData;

use crate::cogs::{
    ActiveLineageSampler, CoalescenceSampler, DispersalSampler, EventSampler, Habitat,
    LineageReference, LineageStore,
};

#[derive(TypedBuilder, Debug)]
#[cfg_attr(feature = "cuda", derive(RustToCuda, LendToCuda))]
#[cfg_attr(feature = "cuda", r2cBound(H: rust_cuda::common::RustToCuda))]
#[cfg_attr(feature = "cuda", r2cBound(D: rust_cuda::common::RustToCuda))]
#[cfg_attr(feature = "cuda", r2cBound(R: rustacuda_core::DeviceCopy))]
#[cfg_attr(feature = "cuda", r2cBound(S: rust_cuda::common::RustToCuda))]
#[cfg_attr(feature = "cuda", r2cBound(C: rust_cuda::common::RustToCuda))]
#[cfg_attr(feature = "cuda", r2cBound(E: rust_cuda::common::RustToCuda))]
#[cfg_attr(feature = "cuda", r2cBound(A: rust_cuda::common::RustToCuda))]
pub struct Simulation<
    H: Habitat,
    D: DispersalSampler<H>,
    R: LineageReference<H>,
    S: LineageStore<H, R>,
    C: CoalescenceSampler<H, R, S>,
    E: EventSampler<H, D, R, S, C>,
    A: ActiveLineageSampler<H, D, R, S, C, E>,
> {
    #[builder(default = 0.0_f64, setter(skip))]
    pub(super) time: f64,
    pub(super) speciation_probability_per_generation: f64,
    #[cfg_attr(feature = "cuda", r2cEmbed)]
    pub(super) habitat: H,
    #[cfg_attr(feature = "cuda", r2cEmbed)]
    pub(super) dispersal_sampler: D,
    pub(super) lineage_reference: PhantomData<R>,
    #[cfg_attr(feature = "cuda", r2cEmbed)]
    pub(super) lineage_store: S,
    #[cfg_attr(feature = "cuda", r2cEmbed)]
    pub(super) coalescence_sampler: C,
    #[cfg_attr(feature = "cuda", r2cEmbed)]
    pub(super) event_sampler: E,
    #[cfg_attr(feature = "cuda", r2cEmbed)]
    pub(super) active_lineage_sampler: A,
}

impl<
        H: Habitat,
        D: DispersalSampler<H>,
        R: LineageReference<H>,
        S: LineageStore<H, R>,
        C: CoalescenceSampler<H, R, S>,
        E: EventSampler<H, D, R, S, C>,
        A: ActiveLineageSampler<H, D, R, S, C, E>,
    > Simulation<H, D, R, S, C, E, A>
{
    pub fn with_mut_split_active_lineage_sampler<
        's,
        Q,
        F: FnOnce(
            &'s mut A,
            &mut super::partial::active_lineager_sampler::PartialSimulation<'s, H, D, R, S, C, E>,
        ) -> Q,
    >(
        &'s mut self,
        func: F,
    ) -> Q {
        let mut simulation = super::partial::active_lineager_sampler::PartialSimulation {
            speciation_probability_per_generation: &self.speciation_probability_per_generation,
            habitat: &self.habitat,
            dispersal_sampler: &self.dispersal_sampler,
            lineage_reference: &self.lineage_reference,
            lineage_store: &mut self.lineage_store,
            coalescence_sampler: &self.coalescence_sampler,
            event_sampler: &self.event_sampler,
        };

        func(&mut self.active_lineage_sampler, &mut simulation)
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

    pub fn speciation_probability_per_generation(&self) -> f64 {
        self.speciation_probability_per_generation
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
}
