use core::num::Wrapping;

use crate::{
    cogs::{
        event_sampler::EventHandler, ActiveLineageSampler, CoalescenceSampler, DispersalSampler,
        EmigrationExit, EventSampler, Habitat, ImmigrationEntry, LineageReference, LineageStore,
        RngCore, SpeciationProbability, TurnoverRate,
    },
    event::{DispersalEvent, SpeciationEvent},
    reporter::Reporter,
    simulation::Simulation,
};

#[allow(clippy::type_complexity)]
pub fn simulate_and_report_local_step_or_finish<
    H: Habitat,
    G: RngCore,
    R: LineageReference<H>,
    S: LineageStore<H, R>,
    X: EmigrationExit<H, G, R, S>,
    D: DispersalSampler<H, G>,
    C: CoalescenceSampler<H, R, S>,
    T: TurnoverRate<H>,
    N: SpeciationProbability<H>,
    E: EventSampler<H, G, R, S, X, D, C, T, N>,
    I: ImmigrationEntry,
    A: ActiveLineageSampler<H, G, R, S, X, D, C, T, N, E, I>,
    P: Reporter,
>(
    simulation: &mut Simulation<H, G, R, S, X, D, C, T, N, E, I, A>,
    reporter: &mut P,
) -> bool {
    let mut emigration = false;

    let should_continue = simulation.with_mut_split_active_lineage_sampler_and_rng(
        |active_lineage_sampler, simulation, rng| {
            // Fetch the next `chosen_lineage` to be simulated with its
            // `dispersal_origin` and `event_time`
            active_lineage_sampler.with_next_active_lineage_and_event_time(
                simulation,
                rng,
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
        simulation.migration_balance += Wrapping(1_u64);
    }

    should_continue
}
