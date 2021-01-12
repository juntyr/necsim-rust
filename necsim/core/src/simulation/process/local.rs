use crate::{
    cogs::{
        ActiveLineageSampler, CoalescenceSampler, DispersalSampler, EmigrationExit, EventSampler,
        Habitat, ImmigrationEntry, LineageReference, LineageStore, RngCore, SpeciationProbability,
    },
    event::EventType,
    reporter::Reporter,
    simulation::Simulation,
};

pub fn simulate_and_report_local_step_or_finish<
    H: Habitat,
    G: RngCore,
    N: SpeciationProbability<H>,
    D: DispersalSampler<H, G>,
    R: LineageReference<H>,
    S: LineageStore<H, R>,
    X: EmigrationExit<H, G, N, D, R, S>,
    C: CoalescenceSampler<H, R, S>,
    E: EventSampler<H, G, N, D, R, S, X, C>,
    I: ImmigrationEntry,
    A: ActiveLineageSampler<H, G, N, D, R, S, X, C, E, I>,
    P: Reporter,
>(
    simulation: &mut Simulation<H, G, N, D, R, S, X, C, E, I, A>,
    reporter: &mut P,
) -> bool {
    simulation.with_mut_split_active_lineage_sampler_and_rng(
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
                        .and_then(|event| {
                            // Report the local event
                            reporter.report_event(&event);

                            // In the event of dispersal without coalescence, the lineage remains
                            // active
                            if let EventType::Dispersal {
                                target: dispersal_target,
                                coalescence: None,
                                ..
                            } = event.r#type()
                            {
                                Some(dispersal_target.clone())
                            } else {
                                None
                            }
                        })
                },
            )
        },
    )
}
