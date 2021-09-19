use core::marker::PhantomData;

use necsim_core::{
    cogs::{
        coalescence_sampler::CoalescenceRngSample, event_sampler::EventHandler, Backup,
        CoalescenceSampler, DispersalSampler, EmigrationExit, EventSampler, Habitat, RngCore,
        SpeciationProbability, TurnoverRate,
    },
    event::{DispersalEvent, SpeciationEvent},
    lineage::{GlobalLineageReference, Lineage},
    simulation::partial::event_sampler::PartialSimulation,
};
use necsim_core_bond::PositiveF64;

use crate::cogs::{
    coalescence_sampler::independent::IndependentCoalescenceSampler,
    lineage_store::independent::IndependentLineageStore,
};

use super::tracking::{MinSpeciationTrackingEventSampler, SpeciationSample};

#[allow(clippy::module_name_repetitions)]
#[derive(Debug)]
#[cfg_attr(feature = "cuda", derive(rust_cuda::common::LendRustToCuda))]
#[cfg_attr(feature = "cuda", r2cBound(H: rust_cuda::common::RustToCuda))]
#[cfg_attr(feature = "cuda", r2cBound(G: rust_cuda::common::RustToCuda))]
#[cfg_attr(feature = "cuda", r2cBound(X: rust_cuda::common::RustToCuda))]
#[cfg_attr(feature = "cuda", r2cBound(D: rust_cuda::common::RustToCuda))]
#[cfg_attr(feature = "cuda", r2cBound(T: rust_cuda::common::RustToCuda))]
#[cfg_attr(feature = "cuda", r2cBound(N: rust_cuda::common::RustToCuda))]
pub struct IndependentEventSampler<
    H: Habitat,
    G: RngCore,
    X: EmigrationExit<H, G, GlobalLineageReference, IndependentLineageStore<H>>,
    D: DispersalSampler<H, G>,
    T: TurnoverRate<H>,
    N: SpeciationProbability<H>,
> {
    #[cfg_attr(feature = "cuda", r2cEmbed(
        Option<rust_cuda::utils::device_copy::SafeDeviceCopyWrapper<SpeciationSample>>
    ))]
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
    #[allow(clippy::type_complexity)]
    #[inline]
    fn sample_event_for_lineage_at_event_time_or_emigrate<
        Q,
        Aux,
        FS: FnOnce(SpeciationEvent, Aux) -> Q,
        FD: FnOnce(DispersalEvent, Aux) -> Q,
        FE: FnOnce(Aux) -> Q,
    >(
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
        EventHandler {
            speciation,
            dispersal,
            emigration,
        }: EventHandler<FS, FD, FE>,
        auxiliary: Aux,
    ) -> Q {
        use necsim_core::cogs::RngSampler;

        let speciation_sample = rng.sample_uniform();

        SpeciationSample::update_min(
            &mut self.min_spec_sample,
            speciation_sample,
            event_time,
            &dispersal_origin,
        );

        if speciation_sample
            < simulation
                .speciation_probability
                .get_speciation_probability_at_location(
                    dispersal_origin.location(),
                    &simulation.habitat,
                )
        {
            speciation(
                SpeciationEvent {
                    origin: dispersal_origin,
                    prior_time,
                    event_time,
                    global_lineage_reference: global_reference,
                },
                auxiliary,
            )
        } else {
            let dispersal_target = simulation.dispersal_sampler.sample_dispersal_from_location(
                dispersal_origin.location(),
                &simulation.habitat,
                rng,
            );

            // Check for emigration and return None iff lineage emigrated
            if let Some((
                global_reference,
                dispersal_origin,
                dispersal_target,
                prior_time,
                event_time,
            )) = simulation.with_mut_split_emigration_exit(|emigration_exit, simulation| {
                emigration_exit.optionally_emigrate(
                    global_reference,
                    dispersal_origin,
                    dispersal_target,
                    prior_time,
                    event_time,
                    simulation,
                    rng,
                )
            }) {
                let (dispersal_target, interaction) = simulation
                    .coalescence_sampler
                    .sample_interaction_at_location(
                        dispersal_target,
                        &simulation.habitat,
                        &simulation.lineage_store,
                        CoalescenceRngSample::new(rng),
                    );

                dispersal(
                    DispersalEvent {
                        origin: dispersal_origin,
                        prior_time,
                        event_time,
                        global_lineage_reference: global_reference,
                        target: dispersal_target,
                        interaction,
                    },
                    auxiliary,
                )
            } else {
                emigration(auxiliary)
            }
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
