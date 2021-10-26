use necsim_core_bond::{NonNegativeF64, PositiveF64};

use super::{
    CoalescenceSampler, DispersalSampler, EmigrationExit, EventSampler, Habitat, ImmigrationEntry,
    LineageReference, LineageStore, MathsCore, RngCore, SpeciationProbability, TurnoverRate,
};

use crate::{
    landscape::IndexedLocation, lineage::Lineage,
    simulation::partial::active_lineager_sampler::PartialSimulation,
};

#[allow(clippy::inline_always, clippy::inline_fn_without_body)]
#[contract_trait]
pub trait ActiveLineageSampler<
    M: MathsCore,
    H: Habitat<M>,
    G: RngCore<M>,
    R: LineageReference<M, H>,
    S: LineageStore<M, H, R>,
    X: EmigrationExit<M, H, G, R, S>,
    D: DispersalSampler<M, H, G>,
    C: CoalescenceSampler<M, H, R, S>,
    T: TurnoverRate<M, H>,
    N: SpeciationProbability<M, H>,
    E: EventSampler<M, H, G, R, S, X, D, C, T, N>,
    I: ImmigrationEntry<M>,
>: crate::cogs::Backup + core::fmt::Debug
{
    #[must_use]
    fn number_active_lineages(&self) -> usize;

    #[must_use]
    fn get_last_event_time(&self) -> NonNegativeF64;

    #[debug_ensures(match ret {
        Some(_) => {
            self.number_active_lineages() ==
            old(self.number_active_lineages()) - 1
        },
        None => {
            self.number_active_lineages() ==
            old(self.number_active_lineages())
        },
    }, "removes an active lineage if `Some(_)` returned")]
    #[debug_ensures(
        old(self.number_active_lineages()) == 0 -> ret.is_none(),
        "returns `None` of no lineages are left"
    )]
    #[debug_ensures(
        ret.is_some() -> ret.as_ref().unwrap().1 > old(self.get_last_event_time()),
        "event occurs later than last event time"
    )]
    #[debug_ensures(if let Some((ref _lineage, event_time)) = ret {
        self.get_last_event_time() == event_time
    } else { true }, "updates the time of the last event")]
    fn pop_active_lineage_and_event_time<P: FnOnce(PositiveF64) -> bool>(
        &mut self,
        simulation: &mut PartialSimulation<M, H, G, R, S, X, D, C, T, N, E>,
        rng: &mut G,
        early_peek_stop: P,
    ) -> Option<(Lineage, PositiveF64)>;

    #[debug_ensures(
        self.number_active_lineages() == old(self.number_active_lineages()) + 1,
        "adds an active lineage"
    )]
    #[debug_ensures(
        self.get_last_event_time() == old(lineage.last_event_time),
        "updates the time of the last event"
    )]
    fn push_active_lineage(
        &mut self,
        lineage: Lineage,
        simulation: &mut PartialSimulation<M, H, G, R, S, X, D, C, T, N, E>,
        rng: &mut G,
    );

    #[inline]
    fn with_next_active_lineage_and_event_time<
        P: FnOnce(PositiveF64) -> bool,
        F: FnOnce(
            &mut PartialSimulation<M, H, G, R, S, X, D, C, T, N, E>,
            &mut G,
            Lineage,
            PositiveF64,
        ) -> Option<IndexedLocation>,
    >(
        &mut self,
        simulation: &mut PartialSimulation<M, H, G, R, S, X, D, C, T, N, E>,
        rng: &mut G,
        early_peek_stop: P,
        inner: F,
    ) -> bool {
        if let Some((chosen_lineage, event_time)) =
            self.pop_active_lineage_and_event_time(simulation, rng, early_peek_stop)
        {
            let global_reference = chosen_lineage.global_reference.clone();

            if let Some(dispersal_target) = inner(simulation, rng, chosen_lineage, event_time) {
                self.push_active_lineage(
                    Lineage {
                        global_reference,
                        indexed_location: dispersal_target,
                        last_event_time: event_time.into(),
                    },
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
