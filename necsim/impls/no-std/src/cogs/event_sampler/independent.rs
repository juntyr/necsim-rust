use core::marker::PhantomData;

use necsim_core::{
    cogs::{
        Backup, CoalescenceRngSample, CoalescenceSampler, DispersalSampler, EmigrationExit,
        EventSampler, Habitat, MinSpeciationTrackingEventSampler, RngCore, SpeciationProbability,
        SpeciationSample, TurnoverRate,
    },
    event::{DispersalEvent, PackedEvent, SpeciationEvent},
    lineage::{GlobalLineageReference, Lineage},
    simulation::partial::event_sampler::PartialSimulation,
};
use necsim_core_bond::PositiveF64;

use crate::cogs::{
    coalescence_sampler::independent::IndependentCoalescenceSampler,
    lineage_store::independent::IndependentLineageStore,
};

#[allow(clippy::module_name_repetitions)]
#[cfg_attr(feature = "cuda", derive(rust_cuda::common::RustToCudaAsRust))]
#[cfg_attr(feature = "cuda", r2cBound(H: rust_cuda::common::RustToCuda))]
#[cfg_attr(feature = "cuda", r2cBound(G: rust_cuda::common::RustToCuda))]
#[cfg_attr(feature = "cuda", r2cBound(X: rust_cuda::common::RustToCuda))]
#[cfg_attr(feature = "cuda", r2cBound(D: rust_cuda::common::RustToCuda))]
#[cfg_attr(feature = "cuda", r2cBound(T: rust_cuda::common::RustToCuda))]
#[cfg_attr(feature = "cuda", r2cBound(N: rust_cuda::common::RustToCuda))]
#[derive(Debug)]
pub struct IndependentEventSampler<
    H: Habitat,
    G: RngCore,
    X: EmigrationExit<H, G, GlobalLineageReference, IndependentLineageStore<H>>,
    D: DispersalSampler<H, G>,
    T: TurnoverRate<H>,
    N: SpeciationProbability<H>,
> {
    min_spec_sample: Option<SpeciationSample>,
    marker: PhantomData<(H, G, X, D, T, N)>,
}

impl<
        H: Habitat,
        G: RngCore,
        X: EmigrationExit<H, G, GlobalLineageReference, IndependentLineageStore<H>>,
        D: DispersalSampler<H, G>,
        T: TurnoverRate<H>,
        N: SpeciationProbability<H>,
    > Default for IndependentEventSampler<H, G, X, D, T, N>
{
    fn default() -> Self {
        Self {
            min_spec_sample: None,
            marker: PhantomData::<(H, G, X, D, T, N)>,
        }
    }
}

#[contract_trait]
impl<
        H: Habitat,
        G: RngCore,
        X: EmigrationExit<H, G, GlobalLineageReference, IndependentLineageStore<H>>,
        D: DispersalSampler<H, G>,
        T: TurnoverRate<H>,
        N: SpeciationProbability<H>,
    > Backup for IndependentEventSampler<H, G, X, D, T, N>
{
    unsafe fn backup_unchecked(&self) -> Self {
        Self {
            min_spec_sample: self.min_spec_sample.clone(),
            marker: PhantomData::<(H, G, X, D, T, N)>,
        }
    }
}

#[contract_trait]
impl<
        H: Habitat,
        G: RngCore,
        X: EmigrationExit<H, G, GlobalLineageReference, IndependentLineageStore<H>>,
        D: DispersalSampler<H, G>,
        T: TurnoverRate<H>,
        N: SpeciationProbability<H>,
    >
    EventSampler<
        H,
        G,
        GlobalLineageReference,
        IndependentLineageStore<H>,
        X,
        D,
        IndependentCoalescenceSampler<H>,
        T,
        N,
    > for IndependentEventSampler<H, G, X, D, T, N>
{
    #[must_use]
    #[allow(clippy::type_complexity, clippy::shadow_unrelated)]
    #[inline]
    fn sample_event_for_lineage_at_event_time_or_emigrate(
        &mut self,
        Lineage {
            global_reference,
            last_event_time: prior_time,
            indexed_location: dispersal_origin,
        }: Lineage,
        event_time: PositiveF64,
        simulation: &mut PartialSimulation<
            H,
            G,
            GlobalLineageReference,
            IndependentLineageStore<H>,
            X,
            D,
            IndependentCoalescenceSampler<H>,
            T,
            N,
        >,
        rng: &mut G,
    ) -> Option<PackedEvent> {
        use necsim_core::cogs::RngSampler;

        let speciation_sample = rng.sample_uniform();

        let min_speciation_sample =
            SpeciationSample::new(dispersal_origin.clone(), event_time, speciation_sample);

        match &self.min_spec_sample {
            Some(spec_sample) if spec_sample <= &min_speciation_sample => (),
            _ => self.min_spec_sample = Some(min_speciation_sample),
        }

        if speciation_sample
            < simulation
                .speciation_probability
                .get_speciation_probability_at_location(
                    dispersal_origin.location(),
                    &simulation.habitat,
                )
        {
            Some(
                SpeciationEvent {
                    origin: dispersal_origin,
                    prior_time,
                    event_time,
                    global_lineage_reference: global_reference,
                }
                .into(),
            )
        } else {
            let dispersal_target = simulation.dispersal_sampler.sample_dispersal_from_location(
                dispersal_origin.location(),
                &simulation.habitat,
                rng,
            );

            // Check for emigration and return None iff lineage emigrated
            let (global_reference, dispersal_origin, dispersal_target, prior_time, event_time) =
                simulation.with_mut_split_emigration_exit(|emigration_exit, simulation| {
                    emigration_exit.optionally_emigrate(
                        global_reference,
                        dispersal_origin,
                        dispersal_target,
                        prior_time,
                        event_time,
                        simulation,
                        rng,
                    )
                })?;

            let (dispersal_target, interaction) = simulation
                .coalescence_sampler
                .sample_interaction_at_location(
                    dispersal_target,
                    &simulation.habitat,
                    &simulation.lineage_store,
                    CoalescenceRngSample::new(rng),
                );

            Some(
                DispersalEvent {
                    origin: dispersal_origin,
                    prior_time,
                    event_time,
                    global_lineage_reference: global_reference,
                    target: dispersal_target,
                    interaction,
                }
                .into(),
            )
        }
    }
}

impl<
        H: Habitat,
        G: RngCore,
        X: EmigrationExit<H, G, GlobalLineageReference, IndependentLineageStore<H>>,
        D: DispersalSampler<H, G>,
        T: TurnoverRate<H>,
        N: SpeciationProbability<H>,
    >
    MinSpeciationTrackingEventSampler<
        H,
        G,
        GlobalLineageReference,
        IndependentLineageStore<H>,
        X,
        D,
        IndependentCoalescenceSampler<H>,
        T,
        N,
    > for IndependentEventSampler<H, G, X, D, T, N>
{
    fn replace_min_speciation(
        &mut self,
        new: Option<SpeciationSample>,
    ) -> Option<SpeciationSample> {
        // `core::mem::replace()` would be semantically better
        //  - but `clone()` does not spill to local memory
        let old_value = self.min_spec_sample.clone();

        self.min_spec_sample = new;

        old_value
    }
}
