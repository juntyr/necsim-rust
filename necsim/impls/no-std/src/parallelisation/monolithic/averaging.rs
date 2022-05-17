use core::ops::ControlFlow;

use necsim_core::{
    cogs::{
        ActiveLineageSampler, CoalescenceSampler, DispersalSampler, EventSampler, Habitat,
        LocallyCoherentLineageStore, MathsCore, Rng, SpeciationProbability,
        TurnoverRate,
    },
    reporter::Reporter,
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
    G: Rng<M>,
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
    independent_time_slice: PositiveF64,
    local_partition: &mut L,
) -> (Status, NonNegativeF64, u64) {
    // Ensure that the progress bar starts with the expected target
    local_partition.report_progress_sync(simulation.get_balanced_remaining_work().0);

    let mut global_safe_time = NonNegativeF64::zero();

    let mut total_steps = 0_u64;

    while local_partition.reduce_vote_continue(!simulation.is_done()) {
        let next_safe_time = global_safe_time + independent_time_slice;

        let (_, new_steps) = simulation.simulate_incremental_early_stop(
            |_, _, next_event_time| {
                if next_event_time >= next_safe_time {
                    ControlFlow::Break(())
                } else {
                    ControlFlow::Continue(())
                }
            },
            local_partition.get_reporter(),
        );

        total_steps += new_steps;

        // Note: Immigration consistency
        //  If a jumps to b at the same time as b jumps to a,
        //  both are independently enqueued for immigration
        //  (and taken out of the system until then), which
        //  happens at the next global safe time,
        //  i.e. no coalescence occurs

        // Send off the possible emigrant and recieve immigrants
        for mut immigrant in local_partition.migrate_individuals(
            simulation.emigration_exit_mut(),
            MigrationMode::Default,
            MigrationMode::Default,
        ) {
            // Push all immigrations to the next safe point such that they do
            //  not conflict with the independence of the current time slice
            immigrant.event_time = immigrant.event_time.max(next_safe_time);

            simulation.immigration_entry_mut().push(immigrant);
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
                immigrant.event_time = immigrant.event_time.max(next_safe_time);

                simulation.immigration_entry_mut().push(immigrant);
            }
        }

        // Globally advance the simulation to the next safe point
        global_safe_time = next_safe_time.into();
    }

    local_partition.report_progress_sync(0_u64);

    let (global_time, global_steps) = local_partition.reduce_global_time_steps(
        simulation.active_lineage_sampler().get_last_event_time(),
        total_steps,
    );

    (Status::Done, global_time, global_steps)
}
