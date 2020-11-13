use core::marker::PhantomData;

use necsim_core::{
    cogs::{
        CoalescenceSampler, DispersalSampler, EventSampler, Habitat, IncoherentLineageStore,
        LineageReference, RngCore,
    },
    event::{Event, EventType},
    landscape::IndexedLocation,
    simulation::partial::event_sampler::PartialSimulation,
};

use crate::cogs::coalescence_sampler::independent::IndependentCoalescenceSampler;

#[allow(clippy::module_name_repetitions)]
#[cfg_attr(feature = "cuda", derive(RustToCuda))]
#[cfg_attr(feature = "cuda", r2cBound(H: rust_cuda::common::RustToCuda))]
#[cfg_attr(feature = "cuda", r2cBound(G: rust_cuda::common::RustToCuda))]
#[cfg_attr(feature = "cuda", r2cBound(D: rust_cuda::common::RustToCuda))]
#[cfg_attr(feature = "cuda", r2cBound(R: rustacuda_core::DeviceCopy))]
#[cfg_attr(feature = "cuda", r2cBound(S: rust_cuda::common::RustToCuda))]
#[derive(Debug)]
pub struct IndependentEventSampler<
    H: Habitat,
    G: RngCore,
    D: DispersalSampler<H, G>,
    R: LineageReference<H>,
    S: IncoherentLineageStore<H, R>,
>(PhantomData<(H, G, D, R, S)>);

impl<
        H: Habitat,
        G: RngCore,
        D: DispersalSampler<H, G>,
        R: LineageReference<H>,
        S: IncoherentLineageStore<H, R>,
    > Default for IndependentEventSampler<H, G, D, R, S>
{
    fn default() -> Self {
        Self(PhantomData::<(H, G, D, R, S)>)
    }
}

#[contract_trait]
impl<
        H: Habitat,
        G: RngCore,
        D: DispersalSampler<H, G>,
        R: LineageReference<H>,
        S: IncoherentLineageStore<H, R>,
    > EventSampler<H, G, D, R, S, IndependentCoalescenceSampler<H, G, R, S>>
    for IndependentEventSampler<H, G, D, R, S>
{
    #[must_use]
    #[allow(clippy::type_complexity)]
    fn sample_event_for_lineage_at_indexed_location_time(
        &self,
        lineage_reference: R,
        indexed_location: IndexedLocation,
        event_time: f64,
        simulation: &PartialSimulation<H, G, D, R, S, IndependentCoalescenceSampler<H, G, R, S>>,
        rng: &mut G,
    ) -> Event<H, R> {
        use necsim_core::cogs::RngSampler;

        let dispersal_origin = indexed_location;

        let event_type = if rng.sample_event(*simulation.speciation_probability_per_generation) {
            EventType::Speciation
        } else {
            let dispersal_target = simulation
                .dispersal_sampler
                .sample_dispersal_from_location(dispersal_origin.location(), rng);

            let (dispersal_target, optional_coalescence) = simulation
                .coalescence_sampler
                .sample_optional_coalescence_at_location(
                    dispersal_target,
                    simulation.habitat,
                    simulation.lineage_store,
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
