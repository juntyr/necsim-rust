use necsim_core::{
    cogs::{
        CoalescenceSampler, DispersalSampler, EventSampler, Habitat, LineageReference,
        LocallyCoherentLineageStore, PeekableActiveLineageSampler, RngCore, SpeciationProbability,
        TurnoverRate,
    },
    reporter::Reporter,
    simulation::Simulation,
};
use necsim_core_bond::{NonNegativeF64, PositiveF64};

use crate::{
    cogs::{
        emigration_exit::domain::DomainEmigrationExit,
        immigration_entry::buffered::BufferedImmigrationEntry,
    },
    decomposition::Decomposition,
    partitioning::{LocalPartition, MigrationMode},
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
    P: Reporter,
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
    independent_time_slice: PositiveF64,
    local_partition: &mut L,
) -> (NonNegativeF64, u64) {
    // Ensure that the progress bar starts with the expected target
    local_partition.report_progress_sync(simulation.get_balanced_remaining_work().0);

    let mut global_safe_time = NonNegativeF64::zero();

    let mut total_steps = 0_u64;

    while local_partition.reduce_vote_continue(simulation.peek_time_of_next_event().is_some()) {
        let next_safe_time = global_safe_time + independent_time_slice;

        let (_, new_steps) = simulation.simulate_incremental_early_stop(
            |simulation, _| {
                simulation
                    .peek_time_of_next_event()
                    .map_or(true, |next_time| next_time >= next_safe_time)
            },
            local_partition.get_reporter(),
        );

        total_steps += new_steps;

        // Send off the possible emigrant and recieve immigrants
        for mut immigrant in local_partition.migrate_individuals(
            simulation.emigration_exit_mut(),
            MigrationMode::Default,
            MigrationMode::Default,
        ) {
            // Push all immigrations to the next safe point such that they do
            //  not conflict with the independence of the current time slice
            immigrant.event_time = immigrant
                .event_time
                .max(global_safe_time + independent_time_slice);

            simulation.immigration_entry_mut().push(immigrant)
        }

        while local_partition.wait_for_termination() {
            for mut immigrant in local_partition.migrate_individuals(
                &mut core::iter::empty(),
                MigrationMode::Force,
                MigrationMode::Force,
            ) {
                // Push all immigrations to the next safe point such that they
                //  do not conflict with the independence of the current time
                //  slice
                immigrant.event_time = immigrant
                    .event_time
                    .max(global_safe_time + independent_time_slice);

                simulation.immigration_entry_mut().push(immigrant)
            }
        }

        // Globally advance the simulation to the next safe point
        global_safe_time += independent_time_slice.into();
    }

    local_partition.report_progress_sync(0_u64);

    local_partition.reduce_global_time_steps(
        simulation.active_lineage_sampler().get_last_event_time(),
        total_steps,
    )
}
