use crate::cogs::{CoalescenceSampler, DispersalSampler, Habitat, LineageReference, LineageStore};

pub struct PartialSimulation<
    's,
    H: Habitat,
    D: DispersalSampler<H>,
    R: LineageReference<H>,
    S: LineageStore<H, R>,
    C: CoalescenceSampler<H, R, S>,
> {
    pub speciation_probability_per_generation: &'s f64,
    pub habitat: &'s H,
    pub dispersal_sampler: &'s D,
    pub lineage_reference: &'s std::marker::PhantomData<R>,
    pub lineage_store: &'s S,
    pub coalescence_sampler: &'s C,
}
