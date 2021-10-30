use core::{fmt, marker::PhantomData};

use necsim_core_bond::{NonNegativeF64, PositiveF64};

use necsim_core::cogs::{
    Backup, CoalescenceSampler, DispersalSampler, EmigrationExit, EventSampler, Habitat,
    ImmigrationEntry, LineageReference, LocallyCoherentLineageStore, MathsCore, RngCore,
    SpeciationProbability, TurnoverRate,
};

use super::dynamic::stack::DynamicAliasMethodStackSampler;

mod sampler;

#[allow(clippy::module_name_repetitions)]
#[allow(clippy::type_complexity)]
pub struct IndividualAliasActiveLineageSampler<
    M: MathsCore,
    H: Habitat<M>,
    G: RngCore<M>,
    R: LineageReference<M, H>,
    S: LocallyCoherentLineageStore<M, H, R>,
    X: EmigrationExit<M, H, G, R, S>,
    D: DispersalSampler<M, H, G>,
    C: CoalescenceSampler<M, H, R, S>,
    T: TurnoverRate<M, H>,
    N: SpeciationProbability<M, H>,
    E: EventSampler<M, H, G, R, S, X, D, C, T, N>,
    I: ImmigrationEntry<M>,
> {
    alias_sampler: DynamicAliasMethodStackSampler<R>,
    number_active_lineages: usize,
    last_event_time: NonNegativeF64,
    marker: PhantomData<(M, H, G, R, S, X, D, C, T, N, E, I)>,
}

impl<
        M: MathsCore,
        H: Habitat<M>,
        G: RngCore<M>,
        R: LineageReference<M, H>,
        S: LocallyCoherentLineageStore<M, H, R>,
        X: EmigrationExit<M, H, G, R, S>,
        D: DispersalSampler<M, H, G>,
        C: CoalescenceSampler<M, H, R, S>,
        T: TurnoverRate<M, H>,
        N: SpeciationProbability<M, H>,
        E: EventSampler<M, H, G, R, S, X, D, C, T, N>,
        I: ImmigrationEntry<M>,
    > IndividualAliasActiveLineageSampler<M, H, G, R, S, X, D, C, T, N, E, I>
{
    #[must_use]
    pub fn new(habitat: &H, turnover_rate: &T, lineage_store: &S) -> Self {
        let mut alias_sampler = DynamicAliasMethodStackSampler::new();
        let mut number_active_lineages: usize = 0;

        lineage_store
            .iter_local_lineage_references()
            .for_each(|reference| {
                let location = lineage_store[reference.clone()].indexed_location.location();
                let rate = turnover_rate.get_turnover_rate_at_location(location, habitat);

                if let Ok(event_rate) = PositiveF64::new(rate.get()) {
                    alias_sampler.add_push(reference, event_rate);

                    number_active_lineages += 1;
                }
            });

        Self {
            alias_sampler,
            number_active_lineages,
            last_event_time: NonNegativeF64::zero(),
            marker: PhantomData::<(M, H, G, R, S, X, D, C, T, N, E, I)>,
        }
    }
}

impl<
        M: MathsCore,
        H: Habitat<M>,
        G: RngCore<M>,
        R: LineageReference<M, H>,
        S: LocallyCoherentLineageStore<M, H, R>,
        X: EmigrationExit<M, H, G, R, S>,
        D: DispersalSampler<M, H, G>,
        C: CoalescenceSampler<M, H, R, S>,
        T: TurnoverRate<M, H>,
        N: SpeciationProbability<M, H>,
        E: EventSampler<M, H, G, R, S, X, D, C, T, N>,
        I: ImmigrationEntry<M>,
    > fmt::Debug for IndividualAliasActiveLineageSampler<M, H, G, R, S, X, D, C, T, N, E, I>
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("IndividualAliasActiveLineageSampler")
            .field("alias_sampler", &self.alias_sampler)
            .field("number_active_lineages", &self.number_active_lineages)
            .finish()
    }
}

#[contract_trait]
impl<
        M: MathsCore,
        H: Habitat<M>,
        G: RngCore<M>,
        R: LineageReference<M, H>,
        S: LocallyCoherentLineageStore<M, H, R>,
        X: EmigrationExit<M, H, G, R, S>,
        D: DispersalSampler<M, H, G>,
        C: CoalescenceSampler<M, H, R, S>,
        T: TurnoverRate<M, H>,
        N: SpeciationProbability<M, H>,
        E: EventSampler<M, H, G, R, S, X, D, C, T, N>,
        I: ImmigrationEntry<M>,
    > Backup for IndividualAliasActiveLineageSampler<M, H, G, R, S, X, D, C, T, N, E, I>
{
    unsafe fn backup_unchecked(&self) -> Self {
        Self {
            alias_sampler: self.alias_sampler.clone(),
            number_active_lineages: self.number_active_lineages,
            last_event_time: self.last_event_time,
            marker: PhantomData::<(M, H, G, R, S, X, D, C, T, N, E, I)>,
        }
    }
}
