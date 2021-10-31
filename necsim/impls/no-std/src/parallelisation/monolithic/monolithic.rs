use necsim_core::{
    cogs::{
        ActiveLineageSampler, CoalescenceSampler, DispersalSampler, EventSampler, Habitat,
        LineageReference, LocallyCoherentLineageStore, MathsCore, RngCore, SpeciationProbability,
        TurnoverRate,
    },
    reporter::Reporter,
    simulation::Simulation,
};
use necsim_core_bond::NonNegativeF64;

use necsim_partitioning_core::LocalPartition;

use crate::cogs::{
    emigration_exit::never::NeverEmigrationExit, immigration_entry::never::NeverImmigrationEntry,
};

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
    E: EventSampler<M, H, G, R, S, NeverEmigrationExit, D, C, T, N>,
    A: ActiveLineageSampler<M, H, G, R, S, NeverEmigrationExit, D, C, T, N, E, NeverImmigrationEntry>,
    P: Reporter,
    L: LocalPartition<P>,
>(
    simulation: &mut Simulation<
        M,
        H,
        G,
        R,
        S,
        NeverEmigrationExit,
        D,
        C,
        T,
        N,
        E,
        NeverImmigrationEntry,
        A,
    >,
    pause_before: Option<NonNegativeF64>,
    local_partition: &mut L,
) -> (NonNegativeF64, u64) {
    // Ensure that the progress bar starts with the expected target
    local_partition.report_progress_sync(simulation.get_balanced_remaining_work().0);

    let (time, steps) = simulation.simulate_incremental_early_stop(
        |_, _, next_event_time| {
            pause_before.map_or(false, |pause_before| next_event_time >= pause_before)
        },
        local_partition.get_reporter(),
    );

    local_partition.report_progress_sync(simulation.get_balanced_remaining_work().0);

    local_partition.reduce_global_time_steps(time, steps)
}
