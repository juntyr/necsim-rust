use core::num::Wrapping;

use necsim_core_bond::PositiveF64;

use crate::{
    cogs::{
        event_sampler::EventHandler, ActiveLineageSampler, CoalescenceSampler, DispersalSampler,
        EmigrationExit, EventSampler, Habitat, ImmigrationEntry, LineageReference, LineageStore,
        MathsCore, RngCore, SpeciationProbability, TurnoverRate,
    },
    event::{DispersalEvent, SpeciationEvent},
    reporter::Reporter,
    simulation::Simulation,
};

impl<
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
        A: ActiveLineageSampler<M, H, G, R, S, X, D, C, T, N, E, I>,
    > Simulation<M, H, G, R, S, X, D, C, T, N, E, I, A>
{
    pub(in super::super) fn simulate_and_report_local_step_or_early_stop_or_finish<
        P: Reporter,
        F: FnOnce(PositiveF64) -> bool,
    >(
        &mut self,
        reporter: &mut P,
        early_peek: F,
    ) -> bool {
        let mut emigration = false;

        let should_continue = self.with_mut_split_active_lineage_sampler_and_rng(
            |active_lineage_sampler, simulation, rng| {
                // Fetch the next `chosen_lineage` to be simulated with its
                // `dispersal_origin` and `event_time`
                active_lineage_sampler.with_next_active_lineage_and_event_time(
                    simulation,
                    rng,
                    early_peek,
                    |simulation, rng, chosen_lineage, event_time| {
                        // Sample the next `event` for the `chosen_lineage`
                        //  or emigrate the `chosen_lineage`
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
                                        emigration = true;

                                        None
                                    },
                                },
                                reporter,
                            )
                        })
                    },
                )
            },
        );

        if emigration {
            // Emigration increments the migration balance (less local work)
            self.migration_balance += Wrapping(1_u64);
        }

        should_continue
    }
}
