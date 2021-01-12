use core::marker::PhantomData;

use crate::cogs::{
    DispersalSampler, Habitat, LineageReference, LineageStore, RngCore, SpeciationProbability,
};

#[repr(C)]
pub struct PartialSimulation<
    H: Habitat,
    G: RngCore,
    N: SpeciationProbability<H>,
    D: DispersalSampler<H, G>,
    R: LineageReference<H>,
    S: LineageStore<H, R>,
> {
    pub habitat: H,
    speciation_probability: N, // not exposed
    dispersal_sampler: D,      // not exposed
    pub lineage_reference: PhantomData<R>,
    pub lineage_store: S,
    pub rng: PhantomData<G>,
}
