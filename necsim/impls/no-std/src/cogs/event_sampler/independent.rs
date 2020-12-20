use core::marker::PhantomData;

use necsim_core::{
    cogs::{
        CoalescenceSampler, DispersalSampler, EventSampler, Habitat, IncoherentLineageStore,
        LineageReference, MinSpeciationTrackingEventSampler, RngCore, SpeciationSample,
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
> {
    min_spec_sample: Option<SpeciationSample>,
    marker: PhantomData<(H, G, D, R, S)>,
}

impl<
        H: Habitat,
        G: RngCore,
        D: DispersalSampler<H, G>,
        R: LineageReference<H>,
        S: IncoherentLineageStore<H, R>,
    > Default for IndependentEventSampler<H, G, D, R, S>
{
    fn default() -> Self {
        Self {
            min_spec_sample: None,
            marker: PhantomData::<(H, G, D, R, S)>,
        }
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
        &mut self,
        lineage_reference: R,
        indexed_location: IndexedLocation,
        event_time: f64,
        simulation: &PartialSimulation<H, G, D, R, S, IndependentCoalescenceSampler<H, G, R, S>>,
        rng: &mut G,
    ) -> Event<H, R> {
        use necsim_core::cogs::RngSampler;

        let speciation_sample = rng.sample_uniform();

        let min_speciation_sample =
            SpeciationSample::new(indexed_location.clone(), event_time, speciation_sample);

        match &self.min_spec_sample {
            Some(spec_sample) if spec_sample <= &min_speciation_sample => (),
            _ => self.min_spec_sample = Some(min_speciation_sample),
        }

        let dispersal_origin = indexed_location;

        let event_type = if speciation_sample < simulation.speciation_probability_per_generation {
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

impl<
        H: Habitat,
        G: RngCore,
        D: DispersalSampler<H, G>,
        R: LineageReference<H>,
        S: IncoherentLineageStore<H, R>,
    > MinSpeciationTrackingEventSampler<H, G, D, R, S, IndependentCoalescenceSampler<H, G, R, S>>
    for IndependentEventSampler<H, G, D, R, S>
{
    fn replace_min_speciation(
        &mut self,
        new: Option<SpeciationSample>,
    ) -> Option<SpeciationSample> {
        core::mem::replace(&mut self.min_spec_sample, new)
    }
}
