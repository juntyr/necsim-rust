use necsim_core::{
    cogs::{
        ActiveLineageSampler, CoalescenceSampler, DispersalSampler, EmigrationExit, EventSampler,
        Habitat, ImmigrationEntry, LineageReference, LineageStore, MathsCore, RngCore,
        SpeciationProbability, TurnoverRate,
    },
    lineage::Lineage,
};

#[allow(clippy::module_name_repetitions)]
pub trait SingularActiveLineageSampler<
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
>: ActiveLineageSampler<M, H, G, R, S, X, D, C, T, N, E, I>
{
    #[must_use]
    fn replace_active_lineage(&mut self, active_lineage: Option<Lineage>) -> Option<Lineage>;
}
