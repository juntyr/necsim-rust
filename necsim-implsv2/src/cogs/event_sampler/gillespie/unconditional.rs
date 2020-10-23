use necsim_corev2::cogs::{
    CoalescenceSampler, DispersalSampler, EventSampler, Habitat, LineageReference, LineageStore,
};
use necsim_corev2::event::{Event, EventType};
use necsim_corev2::landscape::Location;
use necsim_corev2::rng::Rng;

use super::GillespieEventSampler;

#[allow(clippy::module_name_repetitions)]
pub struct UnconditionalGillespieEventSampler<
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
    > Default for UnconditionalGillespieEventSampler<H, D, R, S, C>
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
    > EventSampler<H, D, R, S, C> for UnconditionalGillespieEventSampler<H, D, R, S, C>
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

#[contract_trait]
impl<
        H: Habitat,
        D: DispersalSampler<H>,
        R: LineageReference<H>,
        S: LineageStore<H, R>,
        C: CoalescenceSampler<H, R, S>,
    > GillespieEventSampler<H, D, R, S, C> for UnconditionalGillespieEventSampler<H, D, R, S, C>
{
    #[must_use]
    fn get_event_rate_at_location(
        &self,
        location: &Location,
        _speciation_probability_per_generation: f64,
        _habitat: &H,
        _dispersal_sampler: &D,
        lineage_store: &S,
        lineage_store_includes_self: bool,
        _coalescence_sampler: &C,
    ) -> f64 {
        #[allow(clippy::cast_precision_loss)]
        let population = (lineage_store
            .get_active_lineages_at_location(location)
            .len()
            + usize::from(!lineage_store_includes_self)) as f64;

        population * 0.5_f64
    }
}
