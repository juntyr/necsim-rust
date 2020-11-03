use core::marker::PhantomData;

use necsim_core::cogs::{
    CoalescenceSampler, DispersalSampler, EventSampler, Habitat, LineageReference, LineageStore,
};
use necsim_core::event::{Event, EventType};
use necsim_core::rng::Rng;
use necsim_core::simulation::partial::event_sampler::PartialSimulation;

#[allow(clippy::module_name_repetitions)]
pub struct UnconditionalEventSampler<
    H: Habitat,
    D: DispersalSampler<H>,
    R: LineageReference<H>,
    S: LineageStore<H, R>,
    C: CoalescenceSampler<H, R, S>,
>(PhantomData<(H, D, R, S, C)>);

impl<
        H: Habitat,
        D: DispersalSampler<H>,
        R: LineageReference<H>,
        S: LineageStore<H, R>,
        C: CoalescenceSampler<H, R, S>,
    > Default for UnconditionalEventSampler<H, D, R, S, C>
{
    fn default() -> Self {
        Self(PhantomData::<(H, D, R, S, C)>)
    }
}

#[contract_trait]
impl<
        H: Habitat,
        D: DispersalSampler<H>,
        R: LineageReference<H>,
        S: LineageStore<H, R>,
        C: CoalescenceSampler<H, R, S>,
    > EventSampler<H, D, R, S, C> for UnconditionalEventSampler<H, D, R, S, C>
{
    #[must_use]
    fn sample_event_for_lineage_at_time(
        &self,
        lineage_reference: R,
        event_time: f64,
        simulation: &PartialSimulation<H, D, R, S, C>,
        rng: &mut impl Rng,
    ) -> Event<H, R> {
        let event_type = if rng.sample_event(*simulation.speciation_probability_per_generation) {
            EventType::Speciation
        } else {
            let dispersal_origin = simulation.lineage_store[lineage_reference.clone()].location();
            let dispersal_target = simulation
                .dispersal_sampler
                .sample_dispersal_from_location(dispersal_origin, rng);

            EventType::Dispersal {
                origin: dispersal_origin.clone(),
                coalescence: simulation
                    .coalescence_sampler
                    .sample_optional_coalescence_at_location(
                        &dispersal_target,
                        simulation.habitat,
                        simulation.lineage_store,
                        rng,
                    ),
                target: dispersal_target,
                _marker: PhantomData::<H>,
            }
        };

        Event::new(event_time, lineage_reference, event_type)
    }
}
