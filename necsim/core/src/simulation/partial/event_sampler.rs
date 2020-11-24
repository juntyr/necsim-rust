use core::marker::PhantomData;

use crate::cogs::{
    CoalescenceSampler, DispersalSampler, Habitat, LineageReference, LineageStore, RngCore,
};

pub struct PartialSimulation<
    's,
    H: Habitat,
    G: RngCore,
    D: DispersalSampler<H, G>,
    R: LineageReference<H>,
    S: LineageStore<H, R>,
    C: CoalescenceSampler<H, G, R, S>,
> {
    pub speciation_probability_per_generation: &'s f64,
    pub habitat: &'s H,
    pub rng: PhantomData<G>,
    pub dispersal_sampler: &'s D,
    pub lineage_reference: &'s PhantomData<R>,
    pub lineage_store: &'s S,
    pub coalescence_sampler: &'s C,
}
