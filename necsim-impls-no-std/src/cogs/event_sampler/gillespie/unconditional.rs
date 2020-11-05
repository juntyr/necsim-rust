use core::marker::PhantomData;

use necsim_core::cogs::{
    CoalescenceSampler, CoherentLineageStore, DispersalSampler, EventSampler, Habitat,
    LineageReference,
};
use necsim_core::event::{Event, EventType};
use necsim_core::landscape::Location;
use necsim_core::rng::Rng;
use necsim_core::simulation::partial::event_sampler::PartialSimulation;

use super::GillespieEventSampler;

#[allow(clippy::module_name_repetitions)]
#[cfg_attr(feature = "cuda", derive(RustToCuda))]
#[cfg_attr(feature = "cuda", r2cBound(H: necsim_cuda::common::RustToCuda))]
#[cfg_attr(feature = "cuda", r2cBound(D: necsim_cuda::common::RustToCuda))]
#[cfg_attr(feature = "cuda", r2cBound(R: necsim_cuda::common::RustToCuda))]
#[cfg_attr(feature = "cuda", r2cBound(S: necsim_cuda::common::RustToCuda))]
#[cfg_attr(feature = "cuda", r2cBound(C: necsim_cuda::common::RustToCuda))]
pub struct UnconditionalGillespieEventSampler<
    H: Habitat,
    D: DispersalSampler<H>,
    R: LineageReference<H>,
    S: CoherentLineageStore<H, R>,
    C: CoalescenceSampler<H, R, S>,
>(PhantomData<(H, D, R, S, C)>);

impl<
        H: Habitat,
        D: DispersalSampler<H>,
        R: LineageReference<H>,
        S: CoherentLineageStore<H, R>,
        C: CoalescenceSampler<H, R, S>,
    > Default for UnconditionalGillespieEventSampler<H, D, R, S, C>
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
        S: CoherentLineageStore<H, R>,
        C: CoalescenceSampler<H, R, S>,
    > EventSampler<H, D, R, S, C> for UnconditionalGillespieEventSampler<H, D, R, S, C>
{
    #[must_use]
    fn sample_event_for_lineage_at_location_time(
        &self,
        lineage_reference: R,
        location: Location,
        event_time: f64,
        simulation: &PartialSimulation<H, D, R, S, C>,
        rng: &mut impl Rng,
    ) -> Event<H, R> {
        let event_type = if rng.sample_event(*simulation.speciation_probability_per_generation) {
            EventType::Speciation
        } else {
            let dispersal_origin = location;
            let dispersal_target = simulation
                .dispersal_sampler
                .sample_dispersal_from_location(&dispersal_origin, rng);

            EventType::Dispersal {
                origin: dispersal_origin,
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

#[contract_trait]
impl<
        H: Habitat,
        D: DispersalSampler<H>,
        R: LineageReference<H>,
        S: CoherentLineageStore<H, R>,
        C: CoalescenceSampler<H, R, S>,
    > GillespieEventSampler<H, D, R, S, C> for UnconditionalGillespieEventSampler<H, D, R, S, C>
{
    #[must_use]
    fn get_event_rate_at_location(
        &self,
        location: &Location,
        simulation: &PartialSimulation<H, D, R, S, C>,
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
