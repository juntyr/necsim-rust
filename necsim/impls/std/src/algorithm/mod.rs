use necsim_core::cogs::{
    ActiveLineageSampler, CoalescenceSampler, EmigrationExit, EventSampler, ImmigrationEntry,
    LineageReference, LineageStore, RngCore,
};

use necsim_impls_no_std::cogs::origin_sampler::pre_sampler::OriginPreSampler;

use crate::scenario::Scenario;

pub mod classical;
pub mod gillespie;
pub mod independent;
pub mod skipping_gillespie;

pub trait Algorithm<
    L: LineageStore<O::Habitat, O::LineageReference>,
    O: Scenario<Self::Rng, L>,
    X: EmigrationExit<O::Habitat, Self::Rng, Self::LineageReference, Self::LineageStore>,
    I: ImmigrationEntry,
>: Sized
{
    type Arguments;

    type Error;

    type Rng: RngCore;
    type LineageReference: LineageReference<O::Habitat>;
    type LineageStore: LineageStore<O::Habitat, Self::LineageReference>;
    type CoalescenceSampler: CoalescenceSampler<
        O::Habitat,
        Self::LineageReference,
        Self::LineageStore,
    >;
    type EventSampler: EventSampler<
        O::Habitat,
        Self::Rng,
        Self::LineageReference,
        Self::LineageStore,
        X,
        O::DispersalSampler,
        Self::CoalescenceSampler,
        O::TurnoverRate,
        O::SpeciationProbability,
    >;
    type ActiveLineageSampler: ActiveLineageSampler<
        O::Habitat,
        Self::Rng,
        Self::LineageReference,
        Self::LineageStore,
        X,
        O::DispersalSampler,
        Self::CoalescenceSampler,
        O::TurnoverRate,
        O::SpeciationProbability,
        Self::EventSampler,
        I,
    >;

    /// # Errors
    ///
    /// Returns a `Self::Error` if initialising the algorithm failed
    fn initialise<P: Iterator<Item = u64>>(
        args: Self::Arguments,
        seed: u64,
        scenario: O,
        pre_sampler: OriginPreSampler<P>,
        emigration_exit: X,
    ) -> Result<Self, Self::Error>;

    #[allow(clippy::type_complexity)]
    fn build(
        self,
    ) -> (
        O::Habitat,
        Self::Rng,
        Self::LineageStore,
        X,
        O::DispersalSampler,
        Self::CoalescenceSampler,
        O::TurnoverRate,
        O::SpeciationProbability,
        Self::EventSampler,
        Self::ActiveLineageSampler,
    );
}
