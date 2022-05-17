use core::{num::NonZeroUsize, ops::ControlFlow};

use necsim_core::{
    cogs::{
        rng::{Exponential, IndexUsize, Lambda, Length},
        ActiveLineageSampler, Backup, CoalescenceSampler, DispersalSampler, DistributionSampler,
        EmigrationExit, GloballyCoherentLineageStore, Habitat, ImmigrationEntry,
        MathsCore, Rng, SpeciationProbability, TurnoverRate,
    },
    lineage::Lineage,
    simulation::partial::active_lineage_sampler::PartialSimulation,
};

use necsim_core_bond::{NonNegativeF64, PositiveF64};

use crate::cogs::event_sampler::gillespie::GillespieEventSampler;

use super::LocationAliasActiveLineageSampler;

#[contract_trait]
impl<
        M: MathsCore,
        H: Habitat<M>,
        G: Rng<M>,
        S: GloballyCoherentLineageStore<M, H>,
        X: EmigrationExit<M, H, G, S>,
        D: DispersalSampler<M, H, G>,
        C: CoalescenceSampler<M, H, S>,
        T: TurnoverRate<M, H>,
        N: SpeciationProbability<M, H>,
        E: GillespieEventSampler<M, H, G, S, X, D, C, T, N>,
        I: ImmigrationEntry<M>,
    > ActiveLineageSampler<M, H, G, S, X, D, C, T, N, E, I>
    for LocationAliasActiveLineageSampler<M, H, G, S, X, D, C, T, N, E, I>
where
    G::Sampler: DistributionSampler<M, G::Generator, G::Sampler, Exponential>,
    G::Sampler: DistributionSampler<M, G::Generator, G::Sampler, IndexUsize>,
{
    type LineageIterator<'a> = impl Iterator<Item = &'a Lineage> where H: 'a, G: 'a, S: 'a, X: 'a, D: 'a, C: 'a, T: 'a, N: 'a, E: 'a, I: 'a;

    #[must_use]
    fn number_active_lineages(&self) -> usize {
        self.number_active_lineages
    }

    #[must_use]
    fn iter_active_lineages_ordered<'a>(
        &'a self,
        habitat: &'a H,
        lineage_store: &'a S,
    ) -> Self::LineageIterator<'a> {
        self.alias_sampler
            .iter_all_events_ordered()
            .flat_map(move |location| {
                lineage_store
                    .get_local_lineage_references_at_location_unordered(location, habitat)
                    .iter()
                    .map(move |local_reference| &lineage_store[local_reference])
            })
    }

    #[must_use]
    fn get_last_event_time(&self) -> NonNegativeF64 {
        self.last_event_time
    }

    #[must_use]
    fn pop_active_lineage_and_event_time<F: FnOnce(PositiveF64) -> ControlFlow<(), ()>>(
        &mut self,
        simulation: &mut PartialSimulation<M, H, G, S, X, D, C, T, N, E>,
        rng: &mut G,
        early_peek_stop: F,
    ) -> Option<(Lineage, PositiveF64)> {
        let total_rate = self.alias_sampler.total_weight();

        if let Ok(lambda) = PositiveF64::new(total_rate.get()) {
            let event_time = self.last_event_time + rng.sample_with::<Exponential>(Lambda(lambda));

            let next_event_time = PositiveF64::max_after(self.last_event_time, event_time);

            if early_peek_stop(next_event_time).is_break() {
                return None;
            }

            self.last_event_time = next_event_time.into();

            // Note: This should always be Some
            let chosen_location = self.alias_sampler.sample_pop(rng)?;

            let lineages_at_location = simulation
                .lineage_store
                .get_local_lineage_references_at_location_unordered(
                    &chosen_location,
                    &simulation.habitat,
                );
            let number_lineages_left_at_location = lineages_at_location.len() - 1;

            // Safety: `lineages_at_location` must be >0 since
            //         `chosen_active_location` can only be selected in that case
            let chosen_lineage_index_at_location = rng.sample_with::<IndexUsize>(Length(unsafe {
                NonZeroUsize::new_unchecked(lineages_at_location.len())
            }));
            // Safety: reference clone is only used to then remove the lineage, which is
            //  owned
            let chosen_lineage_reference = unsafe {
                lineages_at_location[chosen_lineage_index_at_location].backup_unchecked()
            };

            let chosen_lineage = simulation
                .lineage_store
                .extract_lineage_globally_coherent(chosen_lineage_reference, &simulation.habitat);
            self.number_active_lineages -= 1;

            if number_lineages_left_at_location > 0 {
                if let Ok(event_rate_at_location) = PositiveF64::new(
                    // All active lineages which are left, which now excludes
                    //  chosen_lineage_reference, are still in the lineage store
                    simulation
                        .event_sampler
                        .get_event_rate_at_location(
                            &chosen_location,
                            &simulation.habitat,
                            &simulation.lineage_store,
                            &simulation.dispersal_sampler,
                            &simulation.coalescence_sampler,
                            &simulation.turnover_rate,
                            &simulation.speciation_probability,
                        )
                        .get(),
                ) {
                    self.alias_sampler
                        .update_or_add(chosen_location, event_rate_at_location);
                }
            }

            Some((chosen_lineage, next_event_time))
        } else {
            None
        }
    }

    #[debug_requires(
        simulation.lineage_store.get_local_lineage_references_at_location_unordered(
            lineage.indexed_location.location(), &simulation.habitat
        ).len() < (
            simulation.habitat.get_habitat_at_location(
                lineage.indexed_location.location()
            ) as usize
        ), "location has habitat capacity for the lineage"
    )]
    fn push_active_lineage(
        &mut self,
        lineage: Lineage,
        simulation: &mut PartialSimulation<M, H, G, S, X, D, C, T, N, E>,
        _rng: &mut G,
    ) {
        self.last_event_time = lineage.last_event_time;

        let location = lineage.indexed_location.location().clone();

        let _lineage_reference = simulation
            .lineage_store
            .insert_lineage_globally_coherent(lineage, &simulation.habitat);

        if let Ok(event_rate_at_location) = PositiveF64::new(
            // All active lineage references, including lineage_reference,
            //  are now (back) in the lineage store
            simulation
                .event_sampler
                .get_event_rate_at_location(
                    &location,
                    &simulation.habitat,
                    &simulation.lineage_store,
                    &simulation.dispersal_sampler,
                    &simulation.coalescence_sampler,
                    &simulation.turnover_rate,
                    &simulation.speciation_probability,
                )
                .get(),
        ) {
            self.alias_sampler
                .update_or_add(location, event_rate_at_location);

            self.number_active_lineages += 1;
        }
    }
}
