use core::num::Wrapping;

use crate::{
    cogs::{
        ActiveLineageSampler, CoalescenceSampler, DispersalSampler, EmigrationExit, EventSampler,
        Habitat, ImmigrationEntry, LineageStore, MathsCore, RngCore, SpeciationProbability,
        TurnoverRate,
    },
    event::DispersalEvent,
    lineage::{Lineage, MigratingLineage},
    reporter::Reporter,
    simulation::Simulation,
};

impl<
        M: MathsCore,
        H: Habitat<M>,
        G: RngCore<M>,
        S: LineageStore<M, H>,
        X: EmigrationExit<M, H, G, S>,
        D: DispersalSampler<M, H, G>,
        C: CoalescenceSampler<M, H, S>,
        T: TurnoverRate<M, H>,
        N: SpeciationProbability<M, H>,
        E: EventSampler<M, H, G, S, X, D, C, T, N>,
        I: ImmigrationEntry<M>,
        A: ActiveLineageSampler<M, H, G, S, X, D, C, T, N, E, I>,
    > Simulation<M, H, G, S, X, D, C, T, N, E, I, A>
{
    pub(in super::super) fn simulate_and_report_immigration_step<P: Reporter>(
        &mut self,
        reporter: &mut P,

        migrating_lineage: MigratingLineage,
    ) {
        self.with_mut_split_active_lineage_sampler_and_rng_and_migration_balance(
            |active_lineage_sampler, simulation, rng, migration_balance| {
                // Immigration decrements the migration balance (extra external work)
                *migration_balance -= Wrapping(1_u64);

                // Sample the missing coalescence using the random sample generated
                // in the remote sublandscape from where the lineage emigrated
                let (dispersal_target, interaction) = simulation
                    .coalescence_sampler
                    .sample_interaction_at_location(
                        migrating_lineage.dispersal_target,
                        &simulation.habitat,
                        &simulation.lineage_store,
                        migrating_lineage.coalescence_rng_sample,
                    );

                // NOTE: event time rules
                // - event time monotonically increases locally
                // - events from the same individual must have unique event times
                // - events from different individuals should not, but can have the the same
                //   event time - currently this can only occur through partitioning / in the
                //   independent algorithm

                // TODO: inconsistency between monolithic and independent algorithm
                // - a jumps to b at the same time as b jumps to a
                // - independent: no coalescence occurs
                // - monolithic: coalescence will occur, which one depends on which one is
                //   executed first, i.e. random

                // In the event of migration without coalescence, the lineage has
                //  to be added to the active lineage sampler and lineage store
                if !interaction.is_coalescence() {
                    active_lineage_sampler.push_active_lineage(
                        Lineage {
                            global_reference: migrating_lineage.global_reference.clone(),
                            indexed_location: dispersal_target.clone(),
                            last_event_time: migrating_lineage.event_time.into(),
                        },
                        simulation,
                        rng,
                    );
                }

                // Report the migration dispersal event
                reporter.report_dispersal(
                    &DispersalEvent {
                        origin: migrating_lineage.dispersal_origin,
                        prior_time: migrating_lineage.prior_time,
                        event_time: migrating_lineage.event_time,
                        global_lineage_reference: migrating_lineage.global_reference,
                        target: dispersal_target,
                        interaction,
                    }
                    .into(),
                );
            },
        );
    }
}
