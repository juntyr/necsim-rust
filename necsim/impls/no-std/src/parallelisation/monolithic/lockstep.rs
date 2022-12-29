use core::ops::ControlFlow;

use necsim_core::{
    cogs::{
        ActiveLineageSampler, CoalescenceSampler, DispersalSampler, EventSampler, Habitat,
        LocallyCoherentLineageStore, MathsCore, RngCore, SpeciationProbability, TurnoverRate,
    },
    reporter::{NullReporter, Reporter},
    simulation::Simulation,
};
use necsim_core_bond::{NonNegativeF64, PositiveF64};

use necsim_partitioning_core::{LocalPartition, MigrationMode};

use crate::{
    cogs::{
        emigration_exit::domain::DomainEmigrationExit,
        immigration_entry::buffered::BufferedImmigrationEntry,
    },
    decomposition::Decomposition,
    parallelisation::Status,
};

#[allow(clippy::type_complexity)]
pub fn simulate<
    'p,
    M: MathsCore,
    H: Habitat<M>,
    G: RngCore<M>,
    S: LocallyCoherentLineageStore<M, H>,
    D: DispersalSampler<M, H, G>,
    C: CoalescenceSampler<M, H, S>,
    T: TurnoverRate<M, H>,
    N: SpeciationProbability<M, H>,
    O: Decomposition<M, H>,
    E: EventSampler<M, H, G, S, DomainEmigrationExit<M, H, O>, D, C, T, N>,
    A: ActiveLineageSampler<
        M,
        H,
        G,
        S,
        DomainEmigrationExit<M, H, O>,
        D,
        C,
        T,
        N,
        E,
        BufferedImmigrationEntry,
    >,
    P: Reporter,
    L: LocalPartition<'p, P>,
>(
    simulation: &mut Simulation<
        M,
        H,
        G,
        S,
        DomainEmigrationExit<M, H, O>,
        D,
        C,
        T,
        N,
        E,
        BufferedImmigrationEntry,
        A,
    >,
    local_partition: &mut L,
) -> (Status, NonNegativeF64, u64) {
    // Ensure that the progress bar starts with the expected target
    local_partition.report_progress_sync(simulation.get_balanced_remaining_work().0);

    let mut total_steps = 0_u64;

    while local_partition.reduce_vote_continue(!simulation.is_done()) {
        let mut next_local_time = None;

        // Simulate for zero-steps (immediate early stop) without side effects
        //  to peek the next local event time
        simulation.simulate_incremental_early_stop(
            |_, _, next_event_time| {
                next_local_time = Some(next_event_time);

                ControlFlow::BREAK
            },
            &mut NullReporter,
        );

        // Get the next local event time or +inf
        //  (we already know at least one partition has some next event time)
        let next_local_time = next_local_time.unwrap_or_else(PositiveF64::infinity);

        // Note: Immigration consistency
        //  If a wants to jump to b at the same time as b
        //  wants to jumps to a, one of the two is selected
        //  to go first deterministically based on which
        //  partition they belong to, and coalescence occurs.
        //  The non-selected individual's wish to jump is
        //  invalidated as there are now different circum-
        //  stances and since the next event must occur at
        //  a monotonically later time

        // The partition with the next event gets to simulate just the next step
        if let Ok(next_global_time) = local_partition.reduce_vote_min_time(next_local_time) {
            let (_, new_steps) = simulation.simulate_incremental_early_stop(
                |_, _, next_event_time| {
                    if next_event_time > next_global_time {
                        ControlFlow::BREAK
                    } else {
                        ControlFlow::CONTINUE
                    }
                },
                local_partition.get_reporter(),
            );

            total_steps += new_steps;

            // Send off any emigration that might have occurred
            for immigrant in local_partition.migrate_individuals(
                simulation.emigration_exit_mut(),
                MigrationMode::Default,
                MigrationMode::Default,
            ) {
                simulation.immigration_entry_mut().push(immigrant);
            }
        }

        // Synchronise after performing any inter-partition migration
        while local_partition.wait_for_termination() {
            for immigrant in local_partition.migrate_individuals(
                &mut core::iter::empty(),
                MigrationMode::Force,
                MigrationMode::Force,
            ) {
                simulation.immigration_entry_mut().push(immigrant);
            }
        }
    }

    local_partition.report_progress_sync(0_u64);

    let (global_time, global_steps) = local_partition.reduce_global_time_steps(
        simulation.active_lineage_sampler().get_last_event_time(),
        total_steps,
    );

    (Status::Done, global_time, global_steps)
}
