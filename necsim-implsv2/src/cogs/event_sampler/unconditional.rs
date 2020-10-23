use necsim_corev2::cogs::{
    CoalescenceSampler, DispersalSampler, EventSampler, Habitat, LineageReference, LineageStore,
};
use necsim_corev2::event::{Event, EventType};
use necsim_corev2::rng::Rng;

#[allow(clippy::module_name_repetitions)]
pub struct UnconditionalEventSampler<
    H: Habitat,
    D: DispersalSampler<H>,
    R: LineageReference<H>,
    S: LineageStore<H, R>,
    C: CoalescenceSampler<H, R, S>,
>(std::marker::PhantomData<(H, D, R, S, C)>);

impl<
        H: Habitat,
        D: DispersalSampler<H>,
        R: LineageReference<H>,
        S: LineageStore<H, R>,
        C: CoalescenceSampler<H, R, S>,
    > Default for UnconditionalEventSampler<H, D, R, S, C>
{
    fn default() -> Self {
        Self(std::marker::PhantomData::<(H, D, R, S, C)>)
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
        speciation_probability_per_generation: f64,
        habitat: &H,
        dispersal_sampler: &D,
        lineage_store: &S,
        coalescence_sampler: &C,
        rng: &mut impl Rng,
    ) -> Event<H, R> {
        let event_type = if rng.sample_event(speciation_probability_per_generation) {
            EventType::Speciation
        } else {
            let dispersal_origin = lineage_store[lineage_reference.clone()].location();
            let dispersal_target =
                dispersal_sampler.sample_dispersal_from_location(dispersal_origin, rng);

            EventType::Dispersal {
                origin: dispersal_origin.clone(),
                coalescence: coalescence_sampler.sample_optional_coalescence_at_location(
                    &dispersal_target,
                    habitat,
                    lineage_store,
                    rng,
                ),
                target: dispersal_target,
                _marker: std::marker::PhantomData::<H>,
            }
        };

        Event::new(event_time, lineage_reference, event_type)
    }
}
