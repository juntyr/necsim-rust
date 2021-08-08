use core::hash::{Hash, Hasher};

use necsim_core_bond::{ClosedUnitF64, PositiveF64};

use super::{
    CoalescenceSampler, DispersalSampler, EmigrationExit, Habitat, LineageReference, LineageStore,
    RngCore, SpeciationProbability, TurnoverRate,
};
use crate::{
    event::PackedEvent, landscape::IndexedLocation, lineage::Lineage,
    simulation::partial::event_sampler::PartialSimulation,
};

#[allow(clippy::inline_always, clippy::inline_fn_without_body)]
#[contract_trait]
pub trait EventSampler<
    H: Habitat,
    G: RngCore,
    R: LineageReference<H>,
    S: LineageStore<H, R>,
    X: EmigrationExit<H, G, R, S>,
    D: DispersalSampler<H, G>,
    C: CoalescenceSampler<H, R, S>,
    T: TurnoverRate<H>,
    N: SpeciationProbability<H>,
>: crate::cogs::Backup + core::fmt::Debug
{
    #[must_use]
    #[debug_ensures(ret.as_ref().map_or(true, |event: &PackedEvent| {
        event.event_time == event_time
    }), "event occurs at event_time")]
    #[debug_ensures(ret.as_ref().map_or(true, |event: &PackedEvent| {
        event.prior_time == old(lineage.last_event_time)
    }), "event's prior time is prior_time")]
    fn sample_event_for_lineage_at_event_time_or_emigrate(
        &mut self,
        lineage: Lineage,
        event_time: PositiveF64,
        simulation: &mut PartialSimulation<H, G, R, S, X, D, C, T, N>,
        rng: &mut G,
    ) -> Option<PackedEvent>;
}

#[derive(Clone, Debug)]
#[repr(C)]
pub struct SpeciationSample {
    speciation_sample: ClosedUnitF64,
    sample_time: PositiveF64,
    sample_location: IndexedLocation,
}

#[allow(dead_code)]
const EXCESSIVE_OPTION_SPECIATION_SAMPLE_ERROR: [(); 1 - {
    const ASSERT: bool = core::mem::size_of::<Option<SpeciationSample>>()
        == core::mem::size_of::<SpeciationSample>();
    ASSERT
} as usize] = [];

impl SpeciationSample {
    pub fn update_min(
        min_spec_sample: &mut Option<Self>,
        speciation_sample: ClosedUnitF64,
        sample_time: PositiveF64,
        sample_location: &IndexedLocation,
    ) {
        match min_spec_sample {
            Some(min_spec_sample) if min_spec_sample.speciation_sample <= speciation_sample => (),
            _ => {
                *min_spec_sample = Some(Self {
                    speciation_sample,
                    sample_time,
                    sample_location: sample_location.clone(),
                });
            },
        };
    }
}

impl PartialEq for SpeciationSample {
    fn eq(&self, other: &Self) -> bool {
        self.speciation_sample.eq(&other.speciation_sample)
            && self.sample_time.eq(&other.sample_time.get())
            && self.sample_location.eq(&other.sample_location)
    }
}

impl Eq for SpeciationSample {}

impl Hash for SpeciationSample {
    fn hash<S: Hasher>(&self, state: &mut S) {
        self.sample_location.hash(state);
        self.sample_time.hash(state);
        self.speciation_sample.hash(state);
    }
}

#[allow(clippy::module_name_repetitions)]
pub trait MinSpeciationTrackingEventSampler<
    H: Habitat,
    G: RngCore,
    R: LineageReference<H>,
    S: LineageStore<H, R>,
    X: EmigrationExit<H, G, R, S>,
    D: DispersalSampler<H, G>,
    C: CoalescenceSampler<H, R, S>,
    T: TurnoverRate<H>,
    N: SpeciationProbability<H>,
>: EventSampler<H, G, R, S, X, D, C, T, N>
{
    fn replace_min_speciation(&mut self, new: Option<SpeciationSample>)
        -> Option<SpeciationSample>;
}
