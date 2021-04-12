use necsim_core::{
    cogs::{
        CoalescenceSampler, DispersalSampler, EventSampler, Habitat, LineageReference,
        LocallyCoherentLineageStore, PeekableActiveLineageSampler, RngCore, SpeciationProbability,
        TurnoverRate,
    },
    simulation::Simulation,
};

use necsim_impls_no_std::{
    cogs::{
        emigration_exit::domain::DomainEmigrationExit,
        immigration_entry::buffered::BufferedImmigrationEntry,
    },
    decomposition::Decomposition,
    partitioning::{LocalPartition, MigrationMode},
    reporter::ReporterContext,
};

#[allow(clippy::type_complexity)]
pub fn simulate<
    H: Habitat,
    G: RngCore,
    R: LineageReference<H>,
    S: LocallyCoherentLineageStore<H, R>,
    D: DispersalSampler<H, G>,
    C: CoalescenceSampler<H, R, S>,
    T: TurnoverRate<H>,
    N: SpeciationProbability<H>,
    O: Decomposition<H>,
    E: EventSampler<H, G, R, S, DomainEmigrationExit<H, O>, D, C, T, N>,
    A: PeekableActiveLineageSampler<
        H,
        G,
        R,
        S,
        DomainEmigrationExit<H, O>,
        D,
        C,
        T,
        N,
        E,
        BufferedImmigrationEntry,
    >,
    P: ReporterContext,
    L: LocalPartition<P>,
>(
    mut simulation: Simulation<
        H,
        G,
        R,
        S,
        DomainEmigrationExit<H, O>,
        D,
        C,
        T,
        N,
        E,
        BufferedImmigrationEntry,
        A,
    >,
    local_partition: &mut L,
) -> (f64, u64) {
    // Ensure that the progress bar starts with the expected target
    local_partition.report_progress_sync(simulation.get_balanced_remaining_work().0);

    let mut total_steps = 0_u64;

    while local_partition.reduce_vote_continue(simulation.peek_time_of_next_event().is_some()) {
        // Get the next local event time or +inf
        //  (we already know at least one partition has some next event time)
        let next_local_time = simulation
            .peek_time_of_next_event()
            .unwrap_or(f64::INFINITY);

        // The partition with the next event gets to simulate just the next step
        if let Ok(next_global_time) = local_partition.reduce_vote_min_time(next_local_time) {
            let (_, new_steps) = simulation
                .simulate_incremental_until(next_global_time, local_partition.get_reporter());

            total_steps += new_steps;

            // Send off any emigration that might have occurred
            for immigrant in local_partition.migrate_individuals(
                simulation.emigration_exit_mut(),
                MigrationMode::Default,
                MigrationMode::Default,
            ) {
                simulation.immigration_entry_mut().push(immigrant)
            }
        }

        // Synchronise after performing any inter-partition migration
        while local_partition.wait_for_termination() {
            for immigrant in local_partition.migrate_individuals(
                &mut std::iter::empty(),
                MigrationMode::Force,
                MigrationMode::Force,
            ) {
                simulation.immigration_entry_mut().push(immigrant)
            }
        }
    }

    local_partition.report_progress_sync(0_u64);

    local_partition.reduce_global_time_steps(
        simulation.active_lineage_sampler().get_last_event_time(),
        total_steps,
    )
}
