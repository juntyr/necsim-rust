use core::{
    cmp::Ordering,
    hash::{Hash, Hasher},
};

use necsim_core_bond::{ClosedUnitF64, NonNegativeF64, PositiveF64};

use super::{
    CoalescenceSampler, DispersalSampler, EmigrationExit, Habitat, LineageReference, LineageStore,
    RngCore, SpeciationProbability, TurnoverRate,
};
use crate::{
    event::PackedEvent, landscape::IndexedLocation,
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
    #[debug_ensures(if ret.is_none() { simulation.lineage_store.get(
        old(lineage_reference.clone())
    ).is_none() } else { true }, "lineage emigrated if no event is returned")]
    #[debug_ensures(ret.as_ref().map_or(true, |event: &PackedEvent| {
        event.event_time == event_time
    }), "event occurs at event_time")]
    #[debug_ensures(ret.as_ref().map_or(true, |event: &PackedEvent| {
        event.prior_time == prior_time
    }), "event's prior time is prior_time")]
    fn sample_event_for_lineage_at_indexed_location_time_or_emigrate(
        &mut self,
        lineage_reference: R,
        indexed_location: IndexedLocation,
        prior_time: NonNegativeF64,
        event_time: PositiveF64,
        simulation: &mut PartialSimulation<H, G, R, S, X, D, C, T, N>,
        rng: &mut G,
    ) -> Option<PackedEvent>;
}

#[derive(Clone, Debug)]
#[repr(C)]
pub struct SpeciationSample {
    speciation_sample: ClosedUnitF64,
    time: PositiveF64,
    indexed_location: IndexedLocation,
}

#[allow(dead_code)]
const EXCESSIVE_OPTION_SPECIATION_SAMPLE_ERROR: [(); 1 - {
    const ASSERT: bool = core::mem::size_of::<Option<SpeciationSample>>()
        == core::mem::size_of::<SpeciationSample>();
    ASSERT
} as usize] = [];

impl SpeciationSample {
    #[must_use]
    pub fn new(
        indexed_location: IndexedLocation,
        time: PositiveF64,
        speciation_sample: ClosedUnitF64,
    ) -> Self {
        Self {
            speciation_sample,
            time,
            indexed_location,
        }
    }
}

impl PartialEq for SpeciationSample {
    fn eq(&self, other: &Self) -> bool {
        self.speciation_sample.cmp(&other.speciation_sample) == Ordering::Equal
            && self.time.get().total_cmp(&other.time.get()) == Ordering::Equal
            && self.indexed_location == other.indexed_location
    }
}

impl Eq for SpeciationSample {}

impl Hash for SpeciationSample {
    fn hash<S: Hasher>(&self, state: &mut S) {
        self.indexed_location.hash(state);
        self.time.hash(state);
        self.speciation_sample.hash(state);
    }
}

impl PartialOrd for SpeciationSample {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for SpeciationSample {
    fn cmp(&self, other: &Self) -> Ordering {
        self.speciation_sample.cmp(&other.speciation_sample)
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
