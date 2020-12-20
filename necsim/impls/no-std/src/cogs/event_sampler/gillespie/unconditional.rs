use core::marker::PhantomData;

use necsim_core::{
    cogs::{
        CoalescenceSampler, CoherentLineageStore, DispersalSampler, EventSampler, Habitat,
        LineageReference, RngCore,
    },
    event::{Event, EventType},
    landscape::{IndexedLocation, Location},
    simulation::partial::event_sampler::PartialSimulation,
};

use super::GillespieEventSampler;

#[allow(clippy::module_name_repetitions)]
#[derive(Debug)]
pub struct UnconditionalGillespieEventSampler<
    H: Habitat,
    G: RngCore,
    D: DispersalSampler<H, G>,
    R: LineageReference<H>,
    S: CoherentLineageStore<H, R>,
    C: CoalescenceSampler<H, G, R, S>,
>(PhantomData<(H, G, D, R, S, C)>);

impl<
        H: Habitat,
        G: RngCore,
        D: DispersalSampler<H, G>,
        R: LineageReference<H>,
        S: CoherentLineageStore<H, R>,
        C: CoalescenceSampler<H, G, R, S>,
    > Default for UnconditionalGillespieEventSampler<H, G, D, R, S, C>
{
    fn default() -> Self {
        Self(PhantomData::<(H, G, D, R, S, C)>)
    }
}

#[contract_trait]
impl<
        H: Habitat,
        G: RngCore,
        D: DispersalSampler<H, G>,
        R: LineageReference<H>,
        S: CoherentLineageStore<H, R>,
        C: CoalescenceSampler<H, G, R, S>,
    > EventSampler<H, G, D, R, S, C> for UnconditionalGillespieEventSampler<H, G, D, R, S, C>
{
    #[must_use]
    fn sample_event_for_lineage_at_indexed_location_time(
        &mut self,
        lineage_reference: R,
        indexed_location: IndexedLocation,
        event_time: f64,
        simulation: &PartialSimulation<H, G, D, R, S, C>,
        rng: &mut G,
    ) -> Event<H, R> {
        use necsim_core::cogs::RngSampler;

        let dispersal_origin = indexed_location;

        let event_type = if rng.sample_event(simulation.speciation_probability_per_generation) {
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

#[contract_trait]
impl<
        H: Habitat,
        G: RngCore,
        D: DispersalSampler<H, G>,
        R: LineageReference<H>,
        S: CoherentLineageStore<H, R>,
        C: CoalescenceSampler<H, G, R, S>,
    > GillespieEventSampler<H, G, D, R, S, C>
    for UnconditionalGillespieEventSampler<H, G, D, R, S, C>
{
    #[must_use]
    fn get_event_rate_at_location(
        &self,
        location: &Location,
        simulation: &PartialSimulation<H, G, D, R, S, C>,
        lineage_store_includes_self: bool,
    ) -> f64 {
        #[allow(clippy::cast_precision_loss)]
        let population = (simulation
            .lineage_store
            .get_active_lineages_at_location(location)
            .len()
            + usize::from(!lineage_store_includes_self)) as f64;

        population * 0.5_f64
    }
}
