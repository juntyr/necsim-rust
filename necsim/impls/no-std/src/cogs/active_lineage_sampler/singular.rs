use necsim_core::{
    cogs::{
        ActiveLineageSampler, CoalescenceSampler, DispersalSampler, EmigrationExit, EventSampler,
        F64Core, Habitat, ImmigrationEntry, LineageReference, LineageStore, RngCore,
        SpeciationProbability, TurnoverRate,
    },
    lineage::Lineage,
};

#[allow(clippy::module_name_repetitions)]
pub trait SingularActiveLineageSampler<
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
    E: EventSampler<F, H, G, R, S, X, D, C, T, N>,
    I: ImmigrationEntry<F>,
>: ActiveLineageSampler<F, H, G, R, S, X, D, C, T, N, E, I>
{
    #[must_use]
    fn replace_active_lineage(&mut self, active_lineage: Option<Lineage>) -> Option<Lineage>;
}
