use alloc::vec::Vec;
use core::{fmt, marker::PhantomData};

use necsim_core_bond::{NonNegativeF64, PositiveF64};

use necsim_core::cogs::{
    Backup, CoalescenceSampler, DispersalSampler, EmigrationExit, EventSampler, Habitat,
    ImmigrationEntry, LineageReference, LocallyCoherentLineageStore, MathsCore, RngCore,
    SpeciationProbability, TurnoverRate,
};

use crate::cogs::{
    active_lineage_sampler::resuming::ExceptionalLineage,
    origin_sampler::{TrustedOriginSampler, UntrustedOriginSampler},
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
    pub fn init_with_store<'h, O: TrustedOriginSampler<'h, M, Habitat = H>>(
        origin_sampler: O,
        turnover_rate: &T,
    ) -> (S, Self)
    where
        H: 'h,
    {
        let (lineage_store, active_lineage_sampler, _) =
            Self::resume_with_store(origin_sampler, turnover_rate);

        (lineage_store, active_lineage_sampler)
    }

    #[must_use]
    pub fn resume_with_store<'h, O: UntrustedOriginSampler<'h, M, Habitat = H>>(
        mut origin_sampler: O,
        turnover_rate: &T,
    ) -> (S, Self, Vec<ExceptionalLineage>)
    where
        H: 'h,
    {
        #[allow(clippy::cast_possible_truncation)]
        let capacity = origin_sampler.full_upper_bound_size_hint() as usize;

        let mut lineage_store = S::with_capacity(origin_sampler.habitat(), capacity);

        let mut alias_sampler = DynamicAliasMethodStackSampler::new();
        let mut number_active_lineages: usize = 0;
        let mut last_event_time = NonNegativeF64::zero();

        let mut exceptional_lineages = Vec::new();

        while let Some(lineage) = origin_sampler.next() {
            if !origin_sampler
                .habitat()
                .contains(lineage.indexed_location.location())
            {
                exceptional_lineages.push(ExceptionalLineage::OutOfHabitat(lineage));
                continue;
            }

            if lineage.indexed_location.index()
                >= origin_sampler
                    .habitat()
                    .get_habitat_at_location(lineage.indexed_location.location())
            {
                exceptional_lineages.push(ExceptionalLineage::OutOfDeme(lineage));
                continue;
            }

            if lineage_store
                .get_global_lineage_reference_at_indexed_location(
                    &lineage.indexed_location,
                    origin_sampler.habitat(),
                )
                .is_some()
            {
                exceptional_lineages.push(ExceptionalLineage::Coalescence(lineage));
                continue;
            }

            let turnover_rate = turnover_rate.get_turnover_rate_at_location(
                lineage.indexed_location.location(),
                origin_sampler.habitat(),
            );

            if let Ok(event_rate) = PositiveF64::new(turnover_rate.get()) {
                last_event_time = last_event_time.max(lineage.last_event_time);

                let local_reference = lineage_store
                    .insert_lineage_locally_coherent(lineage, origin_sampler.habitat());

                alias_sampler.add_push(local_reference, event_rate);

                number_active_lineages += 1;
            }
        }

        (
            lineage_store,
            Self {
                alias_sampler,
                number_active_lineages,
                last_event_time,
                marker: PhantomData::<(M, H, G, R, S, X, D, C, T, N, E, I)>,
            },
            exceptional_lineages,
        )
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
