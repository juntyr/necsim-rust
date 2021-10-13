use necsim_core_bond::PositiveF64;

use super::{
    CoalescenceSampler, DispersalSampler, EmigrationExit, F64Core, Habitat, LineageReference,
    LineageStore, RngCore, SpeciationProbability, TurnoverRate,
};
use crate::{
    event::{DispersalEvent, SpeciationEvent},
    lineage::Lineage,
    simulation::partial::event_sampler::PartialSimulation,
};

pub struct EventHandler<S, D, E> {
    pub speciation: S,
    pub dispersal: D,
    pub emigration: E,
}

#[allow(clippy::inline_always, clippy::inline_fn_without_body)]
#[contract_trait]
pub trait EventSampler<
    F: F64Core,
    H: Habitat<F>,
    G: RngCore<F>,
    R: LineageReference<F, H>,
    S: LineageStore<F, H, R>,
    X: EmigrationExit<F, H, G, R, S>,
    D: DispersalSampler<F, H, G>,
    C: CoalescenceSampler<F, H, R, S>,
    T: TurnoverRate<F, H>,
    N: SpeciationProbability<F, H>,
>: crate::cogs::Backup + core::fmt::Debug
{
    #[must_use]
    fn sample_event_for_lineage_at_event_time_or_emigrate<
        Q,
        Aux,
        FS: FnOnce(SpeciationEvent, Aux) -> Q,
        FD: FnOnce(DispersalEvent, Aux) -> Q,
        FE: FnOnce(Aux) -> Q,
    >(
        &mut self,
        lineage: Lineage,
        event_time: PositiveF64,
        simulation: &mut PartialSimulation<F, H, G, R, S, X, D, C, T, N>,
        rng: &mut G,
        handler: EventHandler<FS, FD, FE>,
        auxiliary: Aux,
    ) -> Q;
}
