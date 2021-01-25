use super::{
    CoalescenceSampler, DispersalSampler, EmigrationExit, EventSampler, Habitat, ImmigrationEntry,
    LineageReference, LineageStore, RngCore, SpeciationProbability,
};

use crate::{
    landscape::IndexedLocation, lineage::GlobalLineageReference,
    simulation::partial::active_lineager_sampler::PartialSimulation,
};

#[allow(clippy::inline_always, clippy::inline_fn_without_body)]
#[contract_trait]
pub trait ActiveLineageSampler<
    H: Habitat,
    G: RngCore,
    N: SpeciationProbability<H>,
    D: DispersalSampler<H, G>,
    R: LineageReference<H>,
    S: LineageStore<H, R>,
    X: EmigrationExit<H, G, N, D, R, S>,
    C: CoalescenceSampler<H, R, S>,
    E: EventSampler<H, G, N, D, R, S, X, C>,
    I: ImmigrationEntry,
>: core::fmt::Debug
{
    #[must_use]
    fn number_active_lineages(&self) -> usize;

    #[must_use]
    #[debug_ensures(ret >= 0.0_f64, "last event time is non-negative")]
    fn get_time_of_last_event(&self) -> f64;

    #[must_use]
    #[debug_ensures(match ret {
        Some(event_time) => event_time >= 0.0_f64,
        None => true,
    }, "next event time is non-negative")]
    fn peek_time_of_next_event(&mut self, rng: &mut G) -> Option<f64>;

    #[must_use]
    #[allow(clippy::float_cmp)]
    #[debug_ensures(match ret {
        Some(_) => {
            self.number_active_lineages() ==
            old(self.number_active_lineages()) - 1
        },
        None => old(self.number_active_lineages()) == 0,
    }, "removes an active lineage if some left")]
    #[debug_ensures(
        ret.is_some() -> ret.as_ref().unwrap().2 > old(self.get_time_of_last_event()),
        "event occurs later than last event time"
    )]
    // TODO: This property is not satisfied by the independent sampler which caches the lineage
    // #[debug_ensures(match ret {
    //     None => true,
    //     Some((ref reference, ref _location, event_time)) => {
    //         simulation.lineage_store[reference.clone()].time_of_last_event() == event_time
    //     },
    // }, "updates the time of the last event of the returned lineage to the time of the event")]
    #[debug_ensures(match ret {
        None => true,
        Some((ref _reference, ref _location, event_time)) => {
            self.get_time_of_last_event() == event_time
        },
    }, "updates the time of the last event")]
    fn pop_active_lineage_indexed_location_event_time(
        &mut self,
        simulation: &mut PartialSimulation<H, G, N, D, R, S, X, C, E>,
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
        simulation: &mut PartialSimulation<H, G, N, D, R, S, X, C, E>,
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
        simulation: &mut PartialSimulation<H, G, N, D, R, S, X, C, E>,
        rng: &mut G,
    );

    #[inline]
    fn with_next_active_lineage_indexed_location_event_time<
        F: FnOnce(
            &mut PartialSimulation<H, G, N, D, R, S, X, C, E>,
            &mut G,
            R,
            IndexedLocation,
            f64,
        ) -> Option<IndexedLocation>,
    >(
        &mut self,
        simulation: &mut PartialSimulation<H, G, N, D, R, S, X, C, E>,
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
    N: SpeciationProbability<H>,
    D: DispersalSampler<H, G>,
    R: LineageReference<H>,
    S: LineageStore<H, R>,
    X: EmigrationExit<H, G, N, D, R, S>,
    C: CoalescenceSampler<H, R, S>,
    E: EventSampler<H, G, N, D, R, S, X, C>,
    I: ImmigrationEntry,
>: ActiveLineageSampler<H, G, N, D, R, S, X, C, E, I>
{
    #[must_use]
    fn replace_active_lineage(
        &mut self,
        active_lineage_reference: Option<R>,
        habitat: &H,
        lineage_store: &mut S,
    ) -> Option<R>;
}
