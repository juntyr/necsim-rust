use core::marker::PhantomData;

use necsim_core::cogs::{
    CoalescenceSampler, DispersalSampler, EventSampler, Habitat, IncoherentLineageStore,
    LineageReference,
};
use necsim_core::event::{Event, EventType};
use necsim_core::landscape::Location;
use necsim_core::rng::Rng;
use necsim_core::simulation::partial::event_sampler::PartialSimulation;

use crate::cogs::coalescence_sampler::independent::IndependentCoalescenceSampler;

#[allow(clippy::module_name_repetitions)]
#[cfg_attr(feature = "cuda", derive(RustToCuda))]
#[cfg_attr(feature = "cuda", r2cBound(H: rust_cuda::common::RustToCuda))]
#[cfg_attr(feature = "cuda", r2cBound(D: rust_cuda::common::RustToCuda))]
#[cfg_attr(feature = "cuda", r2cBound(R: rustacuda_core::DeviceCopy))]
#[cfg_attr(feature = "cuda", r2cBound(S: rust_cuda::common::RustToCuda))]
pub struct IndependentEventSampler<
    H: Habitat,
    D: DispersalSampler<H>,
    R: LineageReference<H>,
    S: IncoherentLineageStore<H, R>,
>(PhantomData<(H, D, R, S)>);

impl<
        H: Habitat,
        D: DispersalSampler<H>,
        R: LineageReference<H>,
        S: IncoherentLineageStore<H, R>,
    > Default for IndependentEventSampler<H, D, R, S>
{
    fn default() -> Self {
        Self(PhantomData::<(H, D, R, S)>)
    }
}

#[contract_trait]
impl<
        H: Habitat,
        D: DispersalSampler<H>,
        R: LineageReference<H>,
        S: IncoherentLineageStore<H, R>,
    > EventSampler<H, D, R, S, IndependentCoalescenceSampler<H, R, S>>
    for IndependentEventSampler<H, D, R, S>
{
    #[must_use]
    fn sample_event_for_lineage_at_location_time(
        &self,
        lineage_reference: R,
        location: Location,
        event_time: f64,
        simulation: &PartialSimulation<H, D, R, S, IndependentCoalescenceSampler<H, R, S>>,
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
