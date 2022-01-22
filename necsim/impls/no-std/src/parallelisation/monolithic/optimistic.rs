use alloc::vec::Vec;
use core::ops::ControlFlow;

use necsim_core::{
    cogs::{
        backup::BackedUp, ActiveLineageSampler, Backup, CoalescenceSampler, DispersalSampler,
        EventSampler, Habitat, LineageReference, LocallyCoherentLineageStore, MathsCore, RngCore,
        SpeciationProbability, TurnoverRate,
    },
    lineage::MigratingLineage,
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

use super::reporter::BufferingReporterProxy;

#[allow(clippy::type_complexity)]
pub fn simulate<
    M: MathsCore,
    H: Habitat<M>,
    G: RngCore<M>,
    R: LineageReference<M, H>,
    S: LocallyCoherentLineageStore<M, H, R>,
    D: DispersalSampler<M, H, G>,
    C: CoalescenceSampler<M, H, R, S>,
    T: TurnoverRate<M, H>,
    N: SpeciationProbability<M, H>,
    O: Decomposition<M, H>,
    E: EventSampler<M, H, G, R, S, DomainEmigrationExit<M, H, O>, D, C, T, N>,
    A: ActiveLineageSampler<
        M,
        H,
        G,
        R,
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
    L: LocalPartition<P>,
>(
    simulation: &mut Simulation<
        M,
        H,
        G,
        R,
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
    let mut simulation_backup = simulation.backup();

    let mut last_immigrants: Vec<BackedUp<MigratingLineage>> = Vec::new();
    let mut immigrants: Vec<MigratingLineage> = Vec::new();

    let mut total_steps = 0_u64;

    let mut proxy = BufferingReporterProxy::from(local_partition);

    while proxy
        .local_partition()
        .reduce_vote_continue(!simulation.is_done())
    {
        loop {
            let next_safe_time = global_safe_time + independent_time_slice;

            let (_, new_steps) = simulation.simulate_incremental_early_stop(
                |_, _, next_event_time| {
                    if next_event_time >= next_safe_time {
                        ControlFlow::BREAK
                    } else {
                        ControlFlow::CONTINUE
                    }
                },
                &mut proxy,
            );
            total_steps += new_steps;

            // Send off the possible emigrant and recieve immigrants
            immigrants.extend(proxy.local_partition().migrate_individuals(
                simulation.emigration_exit_mut(),
                MigrationMode::Default,
                MigrationMode::Default,
            ));

            while proxy.local_partition().wait_for_termination() {
                immigrants.extend(proxy.local_partition().migrate_individuals(
                    &mut core::iter::empty(),
                    MigrationMode::Force,
                    MigrationMode::Force,
                ));
            }

            immigrants.sort();

            // A global rollback is required if at least one partition received unexpected
            // immigration
            if proxy
                .local_partition()
                .reduce_vote_continue(immigrants != last_immigrants)
            {
                // Roll back the simulation to the last backup, clear out all generated events
                *simulation = simulation_backup.resume();
                proxy.clear_events();

                // Back up the previous immigrating lineages in last_immigrants
                last_immigrants.clear();
                for immigrant in &immigrants {
                    last_immigrants.push(immigrant.backup());
                }

                // Move the immigrating lineages into the simulation's immigration entry
                for immigrant in immigrants.drain(..) {
                    simulation.immigration_entry_mut().push(immigrant);
                }
            } else {
                immigrants.clear();
                last_immigrants.clear();

                break;
            }
        }

        // Globally advance the simulation to the next safe point
        proxy.report_events();
        simulation_backup = simulation.backup();
        global_safe_time += independent_time_slice.into();
    }

    proxy.local_partition().report_progress_sync(0_u64);

    let (global_time, global_steps) = proxy.local_partition().reduce_global_time_steps(
        simulation.active_lineage_sampler().get_last_event_time(),
        total_steps,
    );

    (Status::Done, global_time, global_steps)
}
