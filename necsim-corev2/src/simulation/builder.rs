#![allow(clippy::used_underscore_binding)]
#![allow(clippy::empty_enum)]

use crate::cogs::{
    ActiveLineageSampler, CoalescenceSampler, DispersalSampler, EventSampler, Habitat,
    LineageReference, LineageStore,
};

#[derive(TypedBuilder)]
pub struct Simulation<
    H: Habitat,
    D: DispersalSampler<H>,
    R: LineageReference<H>,
    S: LineageStore<H, R>,
    C: CoalescenceSampler<H, R, S>,
    E: EventSampler<H, D, R, S, C>,
    A: ActiveLineageSampler<H, D, R, S, C, E>,
> {
    pub(super) speciation_probability_per_generation: f64,
    pub(super) habitat: H,
    pub(super) dispersal_sampler: D,
    pub(super) lineage_reference: std::marker::PhantomData<R>,
    pub(super) lineage_store: S,
    pub(super) coalescence_sampler: C,
    pub(super) event_sampler: E,
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
