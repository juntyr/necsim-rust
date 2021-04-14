use alloc::vec::Vec;

use necsim_core::{
    cogs::{
        BackedUp, Backup, CoalescenceSampler, DispersalSampler, EventSampler, Habitat,
        LineageReference, LocallyCoherentLineageStore, PeekableActiveLineageSampler, RngCore,
        SpeciationProbability, TurnoverRate,
    },
    lineage::MigratingLineage,
    simulation::Simulation,
};
use necsim_core_bond::PositiveF64;

use crate::{
    cogs::{
        emigration_exit::domain::DomainEmigrationExit,
        immigration_entry::buffered::BufferedImmigrationEntry,
    },
    decomposition::Decomposition,
    partitioning::{LocalPartition, MigrationMode},
    reporter::ReporterContext,
};

use super::reporter::BufferingReporterProxy;

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
    independent_time_slice: PositiveF64,
    local_partition: &mut L,
) -> (f64, u64) {
    // Ensure that the progress bar starts with the expected target
    local_partition.report_progress_sync(simulation.get_balanced_remaining_work().0);

    let mut global_safe_time = 0.0_f64;
    let mut simulation_backup = simulation.backup();

    let mut last_immigrants: Vec<BackedUp<MigratingLineage>> = Vec::new();
    let mut immigrants: Vec<MigratingLineage> = Vec::new();

    let mut total_steps = 0_u64;

    let mut proxy = BufferingReporterProxy::from(local_partition);

    while proxy
        .local_partition()
        .reduce_vote_continue(simulation.peek_time_of_next_event().is_some())
    {
        loop {
            let (_, new_steps) = simulation.simulate_incremental_until_before(
                global_safe_time + independent_time_slice.get(),
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
                ))
            }

            immigrants.sort();

            // A global rollback is required if at least one partition received unexpected
            // immigration
            if proxy
                .local_partition()
                .reduce_vote_continue(immigrants != last_immigrants)
            {
                // Roll back the simulation to the last backup, clear out all generated events
                simulation = simulation_backup.resume();
                proxy.clear_events();

                // Back up the previous immigrating lineages in last_immigrants
                last_immigrants.clear();
                for immigrant in &immigrants {
                    last_immigrants.push(immigrant.backup())
                }

                // Move the immigrating lineages into the simulation's immigration entry
                for immigrant in immigrants.drain(..) {
                    simulation.immigration_entry_mut().push(immigrant)
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
        global_safe_time += independent_time_slice.get();
    }

    proxy.local_partition().report_progress_sync(0_u64);

    proxy.local_partition().reduce_global_time_steps(
        simulation.active_lineage_sampler().get_last_event_time(),
        total_steps,
    )
}
