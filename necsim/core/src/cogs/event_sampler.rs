use core::{
    cmp::Ordering,
    hash::{Hash, Hasher},
    num::NonZeroU64,
};

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
    #[debug_requires(event_time >= 0.0_f64, "event time is non-negative")]
    #[debug_ensures(if ret.is_none() { simulation.lineage_store.get(
        old(lineage_reference.clone())
    ).is_none() } else { true }, "lineage emigrated if no event is returned")]
    #[debug_ensures(ret.as_ref().map_or(true, |event: &PackedEvent| {
        event.event_time.to_bits() == event_time.to_bits()
    }), "event occurs at event_time")]
    #[debug_ensures(ret.as_ref().map_or(true, |event: &PackedEvent| {
        event.prior_time.to_bits() == prior_time.to_bits()
    }), "event's prior time is prior_time")]
    fn sample_event_for_lineage_at_indexed_location_time_or_emigrate(
        &mut self,
        lineage_reference: R,
        indexed_location: IndexedLocation,
        prior_time: f64,
        event_time: f64,
        simulation: &mut PartialSimulation<H, G, R, S, X, D, C, T, N>,
        rng: &mut G,
    ) -> Option<PackedEvent>;
}

// The time of a speciation sample can be stored as a NonZeroU64 as:
// - an f64 can be stored as its u64 binary representation
// - a speciation sample is generated at an event time
// - every event must happen at a strictly greater time than the previous one
// - the simulation starts at time 0.0

#[derive(Clone, Debug)]
#[cfg_attr(feature = "cuda", derive(DeviceCopy))]
pub struct SpeciationSample {
    indexed_location: IndexedLocation,
    time: NonZeroU64,
    speciation_sample: f64,
}

impl SpeciationSample {
    #[must_use]
    #[debug_requires(time > 0.0_f64, "time must be positive")]
    #[debug_requires(
        (0.0_f64..=1.0_f64).contains(&speciation_sample),
        "speciation_sample is a probability"
    )]
    pub fn new(indexed_location: IndexedLocation, time: f64, speciation_sample: f64) -> Self {
        // From the precondition time > 0.0_f64, we know that time =/= 0.0_f64
        //  i.e. time =/= 0_u64

        Self {
            indexed_location,
            time: unsafe { NonZeroU64::new_unchecked(time.to_bits()) },
            speciation_sample,
        }
    }
}

impl PartialEq for SpeciationSample {
    fn eq(&self, other: &Self) -> bool {
        self.speciation_sample.total_cmp(&other.speciation_sample) == Ordering::Equal
            && f64::from_bits(self.time.get()).total_cmp(&f64::from_bits(other.time.get()))
                == Ordering::Equal
            && self.indexed_location == other.indexed_location
    }
}

impl Eq for SpeciationSample {}

impl Hash for SpeciationSample {
    fn hash<S: Hasher>(&self, state: &mut S) {
        self.indexed_location.hash(state);
        self.time.hash(state);
        self.speciation_sample.to_bits().hash(state);
    }
}

impl PartialOrd for SpeciationSample {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.speciation_sample.partial_cmp(&other.speciation_sample)
    }
}

impl Ord for SpeciationSample {
    fn cmp(&self, other: &Self) -> Ordering {
        self.speciation_sample.total_cmp(&other.speciation_sample)
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
