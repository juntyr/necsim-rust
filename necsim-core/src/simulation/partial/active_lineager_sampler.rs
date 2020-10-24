use crate::cogs::{
    CoalescenceSampler, DispersalSampler, EventSampler, Habitat, LineageReference, LineageStore,
};

pub struct PartialSimulation<
    's,
    H: Habitat,
    D: DispersalSampler<H>,
    R: LineageReference<H>,
    S: LineageStore<H, R>,
    C: CoalescenceSampler<H, R, S>,
    E: EventSampler<H, D, R, S, C>,
> {
    pub speciation_probability_per_generation: &'s f64,
    pub habitat: &'s H,
    pub dispersal_sampler: &'s D,
    pub lineage_reference: &'s std::marker::PhantomData<R>,
    pub lineage_store: &'s mut S,
    pub coalescence_sampler: &'s C,
    pub event_sampler: &'s E,
}

impl<
        's,
        H: Habitat,
        D: DispersalSampler<H>,
        R: LineageReference<H>,
        S: LineageStore<H, R>,
        C: CoalescenceSampler<H, R, S>,
        E: EventSampler<H, D, R, S, C>,
    > PartialSimulation<'s, H, D, R, S, C, E>
{
    pub fn with_split_event_sampler<
        Q,
        F: FnOnce(&'s E, &super::event_sampler::PartialSimulation<'s, H, D, R, S, C>) -> Q,
    >(
        &'s self,
        func: F,
    ) -> Q {
        let simulation = super::event_sampler::PartialSimulation {
            speciation_probability_per_generation: self.speciation_probability_per_generation,
            habitat: self.habitat,
            dispersal_sampler: self.dispersal_sampler,
            lineage_reference: self.lineage_reference,
            lineage_store: self.lineage_store,
            coalescence_sampler: self.coalescence_sampler,
        };

        func(self.event_sampler, &simulation)
    }
}
