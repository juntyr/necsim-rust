use super::{
    CoalescenceSampler, DispersalSampler, EmigrationExit, EventSampler, Habitat, ImmigrationEntry,
    LineageReference, LineageStore, RngCore, SpeciationProbability, TurnoverRate,
};

use crate::{
    landscape::IndexedLocation,
    lineage::{GlobalLineageReference, Lineage},
    simulation::partial::active_lineager_sampler::PartialSimulation,
};

#[allow(clippy::inline_always, clippy::inline_fn_without_body)]
#[contract_trait]
pub trait ActiveLineageSampler<
    H: Habitat,
    G: RngCore,
    R: LineageReference<H>,
    S: LineageStore<H, R>,
    X: EmigrationExit<H, G, R, S>,
    D: DispersalSampler<H, G>,
    C: CoalescenceSampler<H, R, S>,
    T: TurnoverRate<H>,
    N: SpeciationProbability<H>,
    E: EventSampler<H, G, R, S, X, D, C, T, N>,
    I: ImmigrationEntry,
>: crate::cogs::Backup + core::fmt::Debug
{
    #[must_use]
    fn number_active_lineages(&self) -> usize;

    #[must_use]
    #[debug_ensures(ret >= 0.0_f64, "last event time is non-negative")]
    fn get_last_event_time(&self) -> f64;

    #[must_use]
    #[debug_ensures(match ret {
        Some(_) => {
            self.number_active_lineages() ==
            old(self.number_active_lineages()) - 1
        },
        None => old(self.number_active_lineages()) == 0,
    }, "removes an active lineage if some left")]
    #[debug_ensures(
        ret.is_some() -> ret.as_ref().unwrap().2 > old(self.get_last_event_time()),
        "event occurs later than last event time"
    )]
    #[debug_ensures(if let Some((ref reference, ref _location, event_time)) = ret {
        simulation.lineage_store.get(reference.clone()).map_or(true, |lineage| {
            lineage.last_event_time().to_bits() == event_time.to_bits()
        })
    } else { true } , "updates the time of the last event of the returned lineage to event time")]
    #[debug_ensures(if let Some((ref _reference, ref _location, event_time)) = ret {
        self.get_last_event_time().to_bits() == event_time.to_bits()
    } else { true }, "updates the time of the last event")]
    fn pop_active_lineage_indexed_location_event_time(
        &mut self,
        simulation: &mut PartialSimulation<H, G, R, S, X, D, C, T, N, E>,
        rng: &mut G,
    ) -> Option<(R, IndexedLocation, f64)>;

    #[debug_requires(time >= 0.0_f64, "time is non-negative")]
    #[debug_ensures(
        self.number_active_lineages() == old(self.number_active_lineages()) + 1,
        "adds an active lineage"
    )]
    fn push_active_lineage_to_indexed_location(
        &mut self,
        lineage_reference: R,
        indexed_location: IndexedLocation,
        time: f64,
        simulation: &mut PartialSimulation<H, G, R, S, X, D, C, T, N, E>,
        rng: &mut G,
    );

    #[debug_requires(time >= 0.0_f64, "time is non-negative")]
    #[debug_ensures(
        self.number_active_lineages() == old(self.number_active_lineages()) + 1,
        "adds an active lineage"
    )]
    fn insert_new_lineage_to_indexed_location(
        &mut self,
        global_reference: GlobalLineageReference,
        indexed_location: IndexedLocation,
        time: f64,
        simulation: &mut PartialSimulation<H, G, R, S, X, D, C, T, N, E>,
        rng: &mut G,
    );

    #[inline]
    fn with_next_active_lineage_indexed_location_event_time<
        F: FnOnce(
            &mut PartialSimulation<H, G, R, S, X, D, C, T, N, E>,
            &mut G,
            R,
            IndexedLocation,
            f64,
        ) -> Option<IndexedLocation>,
    >(
        &mut self,
        simulation: &mut PartialSimulation<H, G, R, S, X, D, C, T, N, E>,
        rng: &mut G,
        inner: F,
    ) -> bool {
        if let Some((chosen_lineage, dispersal_origin, event_time)) =
            self.pop_active_lineage_indexed_location_event_time(simulation, rng)
        {
            if let Some(dispersal_target) = inner(
                simulation,
                rng,
                chosen_lineage.clone(),
                dispersal_origin,
                event_time,
            ) {
                self.push_active_lineage_to_indexed_location(
                    chosen_lineage,
                    dispersal_target,
                    event_time,
                    simulation,
                    rng,
                );
            }

            true
        } else {
            false
        }
    }
}

#[allow(clippy::module_name_repetitions)]
pub trait SingularActiveLineageSampler<
    H: Habitat,
    G: RngCore,
    R: LineageReference<H>,
    S: LineageStore<H, R>,
    X: EmigrationExit<H, G, R, S>,
    D: DispersalSampler<H, G>,
    C: CoalescenceSampler<H, R, S>,
    T: TurnoverRate<H>,
    N: SpeciationProbability<H>,
    E: EventSampler<H, G, R, S, X, D, C, T, N>,
    I: ImmigrationEntry,
>: ActiveLineageSampler<H, G, R, S, X, D, C, T, N, E, I>
{
    #[must_use]
    fn replace_active_lineage(&mut self, active_lineage: Option<Lineage>) -> Option<Lineage>;
}

#[allow(clippy::module_name_repetitions)]
pub struct EmptyActiveLineageSamplerError;

#[allow(
    clippy::module_name_repetitions,
    clippy::inline_always,
    clippy::inline_fn_without_body
)]
#[contract_trait]
pub trait PeekableActiveLineageSampler<
    H: Habitat,
    G: RngCore,
    R: LineageReference<H>,
    S: LineageStore<H, R>,
    X: EmigrationExit<H, G, R, S>,
    D: DispersalSampler<H, G>,
    C: CoalescenceSampler<H, R, S>,
    T: TurnoverRate<H>,
    N: SpeciationProbability<H>,
    E: EventSampler<H, G, R, S, X, D, C, T, N>,
    I: ImmigrationEntry,
>: ActiveLineageSampler<H, G, R, S, X, D, C, T, N, E, I>
{
    #[debug_ensures(
        ret.is_err() == (self.number_active_lineages() == 0),
        "only returns Err when no more lineages remain"
    )]
    #[debug_ensures(match ret {
        Ok(event_time) => event_time >= 0.0_f64,
        Err(_) => true,
    }, "next event time is non-negative")]
    fn peek_time_of_next_event(
        &mut self,
        rng: &mut G,
    ) -> Result<f64, EmptyActiveLineageSamplerError>;
}

#[allow(clippy::module_name_repetitions)]
pub trait OptionallyPeekableActiveLineageSampler<
    H: Habitat,
    G: RngCore,
    R: LineageReference<H>,
    S: LineageStore<H, R>,
    X: EmigrationExit<H, G, R, S>,
    D: DispersalSampler<H, G>,
    C: CoalescenceSampler<H, R, S>,
    T: TurnoverRate<H>,
    N: SpeciationProbability<H>,
    E: EventSampler<H, G, R, S, X, D, C, T, N>,
    I: ImmigrationEntry,
>: ActiveLineageSampler<H, G, R, S, X, D, C, T, N, E, I>
{
    fn peek_optional_time_of_next_event(&mut self, rng: &mut G) -> Option<f64>;
}

impl<
        H: Habitat,
        G: RngCore,
        R: LineageReference<H>,
        S: LineageStore<H, R>,
        X: EmigrationExit<H, G, R, S>,
        D: DispersalSampler<H, G>,
        C: CoalescenceSampler<H, R, S>,
        T: TurnoverRate<H>,
        N: SpeciationProbability<H>,
        E: EventSampler<H, G, R, S, X, D, C, T, N>,
        I: ImmigrationEntry,
        A: ActiveLineageSampler<H, G, R, S, X, D, C, T, N, E, I>,
    > OptionallyPeekableActiveLineageSampler<H, G, R, S, X, D, C, T, N, E, I> for A
{
    default fn peek_optional_time_of_next_event(&mut self, _rng: &mut G) -> Option<f64> {
        None
    }
}

impl<
        H: Habitat,
        G: RngCore,
        R: LineageReference<H>,
        S: LineageStore<H, R>,
        X: EmigrationExit<H, G, R, S>,
        D: DispersalSampler<H, G>,
        C: CoalescenceSampler<H, R, S>,
        T: TurnoverRate<H>,
        N: SpeciationProbability<H>,
        E: EventSampler<H, G, R, S, X, D, C, T, N>,
        I: ImmigrationEntry,
        A: PeekableActiveLineageSampler<H, G, R, S, X, D, C, T, N, E, I>,
    > OptionallyPeekableActiveLineageSampler<H, G, R, S, X, D, C, T, N, E, I> for A
{
    fn peek_optional_time_of_next_event(&mut self, rng: &mut G) -> Option<f64> {
        self.peek_time_of_next_event(rng).ok()
    }
}
