use core::marker::PhantomData;

use necsim_core::{
    cogs::{
        CoalescenceRngSample, CoalescenceSampler, DispersalSampler, EmigrationExit, EventSampler,
        Habitat, IncoherentLineageStore, LineageReference, MinSpeciationTrackingEventSampler,
        RngCore, SpeciationProbability, SpeciationSample,
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
#[cfg_attr(feature = "cuda", r2cBound(X: rust_cuda::common::RustToCuda))]
#[derive(Debug)]
pub struct IndependentEventSampler<
    H: Habitat,
    G: RngCore,
    N: SpeciationProbability<H>,
    D: DispersalSampler<H, G>,
    R: LineageReference<H>,
    S: IncoherentLineageStore<H, R>,
    X: EmigrationExit<H, G, N, D, R, S>,
> {
    min_spec_sample: Option<SpeciationSample>,
    marker: PhantomData<(H, G, N, D, R, S, X)>,
}

impl<
        H: Habitat,
        G: RngCore,
        N: SpeciationProbability<H>,
        D: DispersalSampler<H, G>,
        R: LineageReference<H>,
        S: IncoherentLineageStore<H, R>,
        X: EmigrationExit<H, G, N, D, R, S>,
    > Default for IndependentEventSampler<H, G, N, D, R, S, X>
{
    fn default() -> Self {
        Self {
            min_spec_sample: None,
            marker: PhantomData::<(H, G, N, D, R, S, X)>,
        }
    }
}

#[contract_trait]
impl<
        H: Habitat,
        G: RngCore,
        N: SpeciationProbability<H>,
        D: DispersalSampler<H, G>,
        R: LineageReference<H>,
        S: IncoherentLineageStore<H, R>,
        X: EmigrationExit<H, G, N, D, R, S>,
    > EventSampler<H, G, N, D, R, S, X, IndependentCoalescenceSampler<H, R, S>>
    for IndependentEventSampler<H, G, N, D, R, S, X>
{
    #[must_use]
    #[allow(clippy::type_complexity)]
    #[allow(clippy::shadow_unrelated)] // https://github.com/rust-lang/rust-clippy/issues/5455
    #[inline]
    fn sample_event_for_lineage_at_indexed_location_time_or_emigrate(
        &mut self,
        lineage_reference: R,
        indexed_location: IndexedLocation,
        event_time: f64,
        simulation: &mut PartialSimulation<
            H,
            G,
            N,
            D,
            R,
            S,
            X,
            IndependentCoalescenceSampler<H, R, S>,
        >,
        rng: &mut G,
    ) -> Option<Event> {
        use necsim_core::cogs::RngSampler;

        let speciation_sample = rng.sample_uniform();

        let min_speciation_sample =
            SpeciationSample::new(indexed_location.clone(), event_time, speciation_sample);

        match &self.min_spec_sample {
            Some(spec_sample) if spec_sample <= &min_speciation_sample => (),
            _ => self.min_spec_sample = Some(min_speciation_sample),
        }

        let dispersal_origin = indexed_location;

        let (event_type, lineage_reference, dispersal_origin, event_time) = if speciation_sample
            < simulation
                .speciation_probability
                .get_speciation_probability_at_location(dispersal_origin.location())
        {
            (
                EventType::Speciation,
                lineage_reference,
                dispersal_origin,
                event_time,
            )
        } else {
            let dispersal_target = simulation
                .dispersal_sampler
                .sample_dispersal_from_location(dispersal_origin.location(), rng);

            // Check for emigration and return None iff lineage emigrated
            let (lineage_reference, dispersal_origin, dispersal_target, event_time) = simulation
                .with_mut_split_emigration_exit(|emigration_exit, simulation| {
                    emigration_exit.optionally_emigrate(
                        lineage_reference,
                        dispersal_origin,
                        dispersal_target,
                        event_time,
                        simulation,
                        rng,
                    )
                })?;

            let (dispersal_target, optional_coalescence) = simulation
                .coalescence_sampler
                .sample_optional_coalescence_at_location(
                    dispersal_target,
                    &simulation.habitat,
                    &simulation.lineage_store,
                    CoalescenceRngSample::new(rng),
                );

            (
                EventType::Dispersal {
                    coalescence: optional_coalescence,
                    target: dispersal_target,
                },
                lineage_reference,
                dispersal_origin,
                event_time,
            )
        };

        Some(Event::new(
            dispersal_origin,
            event_time,
            simulation.lineage_store[lineage_reference]
                .global_reference()
                .clone(),
            event_type,
        ))
    }
}

impl<
        H: Habitat,
        G: RngCore,
        N: SpeciationProbability<H>,
        D: DispersalSampler<H, G>,
        R: LineageReference<H>,
        S: IncoherentLineageStore<H, R>,
        X: EmigrationExit<H, G, N, D, R, S>,
    > MinSpeciationTrackingEventSampler<H, G, N, D, R, S, X, IndependentCoalescenceSampler<H, R, S>>
    for IndependentEventSampler<H, G, N, D, R, S, X>
{
    fn replace_min_speciation(
        &mut self,
        new: Option<SpeciationSample>,
    ) -> Option<SpeciationSample> {
        core::mem::replace(&mut self.min_spec_sample, new)
    }
}
