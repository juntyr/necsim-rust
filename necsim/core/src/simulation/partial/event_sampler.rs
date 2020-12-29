use core::marker::PhantomData;

use crate::cogs::{
    CoalescenceSampler, DispersalSampler, Habitat, LineageReference, LineageStore, RngCore,
    SpeciationProbability,
};

#[repr(C)]
pub struct PartialSimulation<
    H: Habitat,
    G: RngCore,
    N: SpeciationProbability<H>,
    D: DispersalSampler<H, G>,
    R: LineageReference<H>,
    S: LineageStore<H, R>,
    C: CoalescenceSampler<H, G, R, S>,
> {
    pub habitat: H,
    pub speciation_probability: N,
    pub dispersal_sampler: D,
    pub lineage_reference: PhantomData<R>,
    pub lineage_store: S,
    pub coalescence_sampler: C,
    pub rng: PhantomData<G>,
}
