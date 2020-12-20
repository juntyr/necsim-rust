use core::marker::PhantomData;

use crate::cogs::{
    CoalescenceSampler, DispersalSampler, Habitat, LineageReference, LineageStore, RngCore,
};

#[repr(C)]
pub struct PartialSimulation<
    H: Habitat,
    G: RngCore,
    D: DispersalSampler<H, G>,
    R: LineageReference<H>,
    S: LineageStore<H, R>,
    C: CoalescenceSampler<H, G, R, S>,
> {
    pub speciation_probability_per_generation: f64,
    pub habitat: H,
    pub dispersal_sampler: D,
    pub lineage_reference: PhantomData<R>,
    pub lineage_store: S,
    pub coalescence_sampler: C,
    pub rng: PhantomData<G>,
}
