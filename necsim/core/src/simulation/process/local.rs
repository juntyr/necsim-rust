use core::num::Wrapping;

use crate::{
    cogs::{
        ActiveLineageSampler, CoalescenceSampler, DispersalSampler, EmigrationExit, EventSampler,
        Habitat, ImmigrationEntry, LineageReference, LineageStore, RngCore, SpeciationProbability,
        TurnoverRate,
    },
    event::{Dispersal, EventType},
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
            active_lineage_sampler.with_next_active_lineage_indexed_location_event_time(
                simulation,
                rng,
                |simulation, rng, chosen_lineage, dispersal_origin, event_time| {
                    // Sample the next `event` for the `chosen_lineage`
                    //  or emigrate the `chosen_lineage`
                    simulation
                        .with_mut_split_event_sampler(|event_sampler, simulation| {
                            event_sampler
                                .sample_event_for_lineage_at_indexed_location_time_or_emigrate(
                                    chosen_lineage,
                                    dispersal_origin,
                                    event_time,
                                    simulation,
                                    rng,
                                )
                        })
                        .map_or_else(
                            || {
                                emigration = true;
                                None
                            },
                            |event| {
                                // Report the local event
                                reporter.report_event(&event);

                                // In the event of dispersal without coalescence, the lineage
                                // remains active
                                if let EventType::Dispersal(Dispersal {
                                    target: dispersal_target,
                                    coalescence: None,
                                    ..
                                }) = event.r#type()
                                {
                                    Some(dispersal_target.clone())
                                } else {
                                    None
                                }
                            },
                        )
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
