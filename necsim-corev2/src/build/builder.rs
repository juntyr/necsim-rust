#![allow(clippy::used_underscore_binding)]
#![allow(clippy::empty_enum)]

use super::stages::{
    CoalescenceSampler, DispersalSampler, EventStage, Habitat, LineageReference, LineageSampler,
    ProbabilityStage,
};

#[derive(TypedBuilder)]
pub struct Simulation<
    H: Habitat,
    D: DispersalSampler<H>,
    R: LineageReference<H>,
    L: LineageSampler<H, R>,
    C: CoalescenceSampler<H, R, L>,
    P: ProbabilityStage<H, D, R, L, C>,
    E: EventStage<H, D, R, L, C, P>,
> {
    pub(super) habitat: H,
    pub(super) dispersal_sampler: D,
    pub(super) lineage_reference: std::marker::PhantomData<R>,
    pub(super) lineage_sampler: L,
    pub(super) coalescence_sampler: C,
    pub(super) probability: P,
    pub(super) event: E,
}
