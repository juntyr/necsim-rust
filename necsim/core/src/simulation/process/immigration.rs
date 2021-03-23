use crate::{
    cogs::{
        ActiveLineageSampler, CoalescenceRngSample, CoalescenceSampler, DispersalSampler,
        EmigrationExit, EventSampler, Habitat, ImmigrationEntry, LineageReference, LineageStore,
        RngCore, SpeciationProbability, TurnoverRate,
    },
    event::{Event, EventType},
    landscape::{IndexedLocation, Location},
    lineage::GlobalLineageReference,
    reporter::Reporter,
    simulation::Simulation,
};

#[allow(clippy::type_complexity)]
pub fn simulate_and_report_immigration_step<
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

    global_reference: GlobalLineageReference,
    dispersal_origin: IndexedLocation,
    dispersal_target: Location,
    migration_event_time: f64,
    coalescence_rng_sample: CoalescenceRngSample,
) {
    simulation.with_mut_split_active_lineage_sampler_and_rng(
        |active_lineage_sampler, simulation, rng| {
            // Sample the missing coalescence using the random sample generated
            // in the remote sublandscape from where the lineage emigrated
            let (dispersal_target, optional_coalescence) = simulation
                .coalescence_sampler
                .sample_optional_coalescence_at_location(
                    dispersal_target,
                    &simulation.habitat,
                    &simulation.lineage_store,
                    coalescence_rng_sample,
                );

            // TODO: How should incrementing time be handled for migration events?
            // - To maintain consistency with the origin, we cannot change the event time
            // - But we also assert that locally events have distinct timestamps

            // In the event of migration without coalescence, the lineage has
            // to be added to the active lineage sampler and lineage store
            if optional_coalescence.is_none() {
                active_lineage_sampler.insert_new_lineage_to_indexed_location(
                    global_reference.clone(),
                    dispersal_target.clone(),
                    migration_event_time,
                    simulation,
                    rng,
                );
            }

            // Report the migration dispersal event
            reporter.report_event(&Event::new(
                dispersal_origin,
                migration_event_time,
                global_reference,
                EventType::Dispersal {
                    target: dispersal_target,
                    coalescence: optional_coalescence,
                },
            ))
        },
    )
}
