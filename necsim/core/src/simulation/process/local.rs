use core::{num::Wrapping, ops::ControlFlow};

use necsim_core_bond::PositiveF64;

use crate::{
    cogs::{
        event_sampler::EventHandler, ActiveLineageSampler, CoalescenceSampler, DispersalSampler,
        EmigrationExit, EventSampler, Habitat, ImmigrationEntry, LineageStore, MathsCore, Rng,
        SpeciationProbability, TurnoverRate,
    },
    event::{DispersalEvent, SpeciationEvent},
    lineage::Lineage,
    reporter::Reporter,
    simulation::Simulation,
};

impl<
        M: MathsCore,
        H: Habitat<M>,
        G: Rng<M>,
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
    #[inline]
    pub(in super::super) fn simulate_and_report_local_step_or_early_stop_or_finish<
        P: Reporter,
        F: FnOnce(PositiveF64) -> ControlFlow<(), ()>,
    >(
        &mut self,
        reporter: &mut P,
        early_peek: F,
    ) -> ControlFlow<(), ()> {
        self.with_mut_split_active_lineage_sampler_and_rng_and_migration_balance(
            |active_lineage_sampler, simulation, rng, migration_balance| {
                // Fetch the next `chosen_lineage` to be simulated at `event_time`
                if let Some((chosen_lineage, event_time)) = active_lineage_sampler
                    .pop_active_lineage_and_event_time(simulation, rng, early_peek)
                {
                    let global_reference = chosen_lineage.global_reference.clone();

                    // Sample the next `event` for the `chosen_lineage`
                    //  or emigrate the `chosen_lineage`
                    let dispersal =
                        simulation.with_mut_split_event_sampler(|event_sampler, simulation| {
                            event_sampler.sample_event_for_lineage_at_event_time_or_emigrate(
                                chosen_lineage,
                                event_time,
                                simulation,
                                rng,
                                EventHandler {
                                    speciation: |event: SpeciationEvent, reporter: &mut P| {
                                        // Report the local speciation event
                                        reporter.report_speciation((&event).into());

                                        None
                                    },
                                    dispersal: |event: DispersalEvent, reporter: &mut P| {
                                        // Report the local dispersal event
                                        reporter.report_dispersal((&event).into());

                                        if event.interaction.is_coalescence() {
                                            None
                                        } else {
                                            Some(event.target)
                                        }
                                    },
                                    emigration: |_| {
                                        // Emigration increments the migration balance
                                        //  (less local work)
                                        *migration_balance += Wrapping(1_u64);

                                        None
                                    },
                                },
                                reporter,
                            )
                        });

                    if let Some(dispersal_target) = dispersal {
                        active_lineage_sampler.push_active_lineage(
                            Lineage {
                                global_reference,
                                indexed_location: dispersal_target,
                                last_event_time: event_time.into(),
                            },
                            simulation,
                            rng,
                        );
                    }

                    ControlFlow::Continue(())
                } else {
                    ControlFlow::Break(())
                }
            },
        )
    }
}
