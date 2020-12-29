use core::marker::PhantomData;

use necsim_core::{
    cogs::{
        CoalescenceSampler, DispersalSampler, EventSampler, Habitat, LineageReference,
        LineageStore, RngCore, SpeciationProbability,
    },
    event::{Event, EventType},
    landscape::IndexedLocation,
    simulation::partial::event_sampler::PartialSimulation,
};

#[allow(clippy::module_name_repetitions)]
#[derive(Debug)]
pub struct UnconditionalEventSampler<
    H: Habitat,
    G: RngCore,
    N: SpeciationProbability<H>,
    D: DispersalSampler<H, G>,
    R: LineageReference<H>,
    S: LineageStore<H, R>,
    C: CoalescenceSampler<H, G, R, S>,
>(PhantomData<(H, G, N, D, R, S, C)>);

impl<
        H: Habitat,
        G: RngCore,
        N: SpeciationProbability<H>,
        D: DispersalSampler<H, G>,
        R: LineageReference<H>,
        S: LineageStore<H, R>,
        C: CoalescenceSampler<H, G, R, S>,
    > Default for UnconditionalEventSampler<H, G, N, D, R, S, C>
{
    fn default() -> Self {
        Self(PhantomData::<(H, G, N, D, R, S, C)>)
    }
}

#[contract_trait]
impl<
        H: Habitat,
        G: RngCore,
        N: SpeciationProbability<H>,
        D: DispersalSampler<H, G>,
        R: LineageReference<H>,
        S: LineageStore<H, R>,
        C: CoalescenceSampler<H, G, R, S>,
    > EventSampler<H, G, N, D, R, S, C> for UnconditionalEventSampler<H, G, N, D, R, S, C>
{
    #[must_use]
    fn sample_event_for_lineage_at_indexed_location_time(
        &mut self,
        lineage_reference: R,
        indexed_location: IndexedLocation,
        event_time: f64,
        simulation: &PartialSimulation<H, G, N, D, R, S, C>,
        rng: &mut G,
    ) -> Event<H, R> {
        use necsim_core::cogs::RngSampler;

        let dispersal_origin = indexed_location;

        let event_type = if rng.sample_event(
            simulation
                .speciation_probability
                .get_speciation_probability_at_location(dispersal_origin.location()),
        ) {
            EventType::Speciation
        } else {
            let dispersal_target = simulation
                .dispersal_sampler
                .sample_dispersal_from_location(dispersal_origin.location(), rng);

            let (dispersal_target, optional_coalescence) = simulation
                .coalescence_sampler
                .sample_optional_coalescence_at_location(
                    dispersal_target,
                    &simulation.habitat,
                    &simulation.lineage_store,
                    rng,
                );

            EventType::Dispersal {
                coalescence: optional_coalescence,
                target: dispersal_target,
                marker: PhantomData::<H>,
            }
        };

        Event::new(dispersal_origin, event_time, lineage_reference, event_type)
    }
}
