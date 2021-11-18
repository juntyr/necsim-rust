use alloc::vec::Vec;
use core::marker::PhantomData;

use necsim_core_bond::{NonNegativeF64, PositiveF64};

use necsim_core::{
    cogs::{
        Backup, CoalescenceSampler, DispersalSampler, EmigrationExit, GloballyCoherentLineageStore,
        Habitat, ImmigrationEntry, LineageReference, MathsCore, RngCore, SpeciationProbability,
        TurnoverRate,
    },
    landscape::Location,
};

use crate::cogs::{
    active_lineage_sampler::resuming::ExceptionalLineage,
    event_sampler::gillespie::GillespieEventSampler,
    origin_sampler::{TrustedOriginSampler, UntrustedOriginSampler},
};

use super::dynamic::indexed::DynamicAliasMethodIndexedSampler;

mod sampler;

#[allow(clippy::module_name_repetitions)]
#[allow(clippy::type_complexity)]
pub struct LocationAliasActiveLineageSampler<
    M: MathsCore,
    H: Habitat<M>,
    G: RngCore<M>,
    R: LineageReference<M, H>,
    S: GloballyCoherentLineageStore<M, H, R>,
    X: EmigrationExit<M, H, G, R, S>,
    D: DispersalSampler<M, H, G>,
    C: CoalescenceSampler<M, H, R, S>,
    T: TurnoverRate<M, H>,
    N: SpeciationProbability<M, H>,
    E: GillespieEventSampler<M, H, G, R, S, X, D, C, T, N>,
    I: ImmigrationEntry<M>,
> {
    alias_sampler: DynamicAliasMethodIndexedSampler<Location>,
    number_active_lineages: usize,
    last_event_time: NonNegativeF64,
    marker: PhantomData<(M, H, G, R, S, X, D, C, T, N, E, I)>,
}

impl<
        M: MathsCore,
        H: Habitat<M>,
        G: RngCore<M>,
        R: LineageReference<M, H>,
        S: GloballyCoherentLineageStore<M, H, R>,
        X: EmigrationExit<M, H, G, R, S>,
        D: DispersalSampler<M, H, G>,
        C: CoalescenceSampler<M, H, R, S>,
        T: TurnoverRate<M, H>,
        N: SpeciationProbability<M, H>,
        E: GillespieEventSampler<M, H, G, R, S, X, D, C, T, N>,
        I: ImmigrationEntry<M>,
    > LocationAliasActiveLineageSampler<M, H, G, R, S, X, D, C, T, N, E, I>
{
    #[must_use]
    pub fn init_with_store<'h, O: TrustedOriginSampler<'h, M, Habitat = H>>(
        origin_sampler: O,
        dispersal_sampler: &D,
        coalescence_sampler: &C,
        turnover_rate: &T,
        speciation_probability: &N,
        event_sampler: &E,
    ) -> (S, Self)
    where
        H: 'h,
    {
        let (lineage_store, active_lineage_sampler, _) = Self::resume_with_store(
            origin_sampler,
            dispersal_sampler,
            coalescence_sampler,
            turnover_rate,
            speciation_probability,
            event_sampler,
        );

        (lineage_store, active_lineage_sampler)
    }

    #[must_use]
    pub fn resume_with_store<'h, O: UntrustedOriginSampler<'h, M, Habitat = H>>(
        mut origin_sampler: O,
        dispersal_sampler: &D,
        coalescence_sampler: &C,
        turnover_rate: &T,
        speciation_probability: &N,
        event_sampler: &E,
    ) -> (S, Self, Vec<ExceptionalLineage>)
    where
        H: 'h,
    {
        #[allow(clippy::cast_possible_truncation)]
        let capacity = origin_sampler.full_upper_bound_size_hint() as usize;

        let mut lineage_store = S::with_capacity(origin_sampler.habitat(), capacity);

        let mut alias_sampler = DynamicAliasMethodIndexedSampler::new();
        let mut last_event_time = NonNegativeF64::zero();

        let mut ordered_active_locations = Vec::new();

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

            if PositiveF64::new(turnover_rate.get()).is_ok() {
                last_event_time = last_event_time.max(lineage.last_event_time);

                match ordered_active_locations.last() {
                    Some(location) if location == lineage.indexed_location.location() => (),
                    _ => ordered_active_locations.push(lineage.indexed_location.location().clone()),
                };

                let _local_reference = lineage_store
                    .insert_lineage_globally_coherent(lineage, origin_sampler.habitat());
            }
        }

        for location in ordered_active_locations {
            if let Ok(event_rate_at_location) = PositiveF64::new(
                event_sampler
                    .get_event_rate_at_location(
                        &location,
                        origin_sampler.habitat(),
                        &lineage_store,
                        dispersal_sampler,
                        coalescence_sampler,
                        turnover_rate,
                        speciation_probability,
                    )
                    .get(),
            ) {
                alias_sampler.update_or_add(location, event_rate_at_location);
            }
        }

        let number_active_lineages = lineage_store
            .iter_active_locations(origin_sampler.habitat())
            .map(|location| {
                lineage_store
                    .get_local_lineage_references_at_location_unordered(
                        &location,
                        origin_sampler.habitat(),
                    )
                    .len()
            })
            .sum();

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
        S: GloballyCoherentLineageStore<M, H, R>,
        X: EmigrationExit<M, H, G, R, S>,
        D: DispersalSampler<M, H, G>,
        C: CoalescenceSampler<M, H, R, S>,
        T: TurnoverRate<M, H>,
        N: SpeciationProbability<M, H>,
        E: GillespieEventSampler<M, H, G, R, S, X, D, C, T, N>,
        I: ImmigrationEntry<M>,
    > core::fmt::Debug for LocationAliasActiveLineageSampler<M, H, G, R, S, X, D, C, T, N, E, I>
{
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        f.debug_struct("LocationAliasActiveLineageSampler")
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
        S: GloballyCoherentLineageStore<M, H, R>,
        X: EmigrationExit<M, H, G, R, S>,
        D: DispersalSampler<M, H, G>,
        C: CoalescenceSampler<M, H, R, S>,
        T: TurnoverRate<M, H>,
        N: SpeciationProbability<M, H>,
        E: GillespieEventSampler<M, H, G, R, S, X, D, C, T, N>,
        I: ImmigrationEntry<M>,
    > Backup for LocationAliasActiveLineageSampler<M, H, G, R, S, X, D, C, T, N, E, I>
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
