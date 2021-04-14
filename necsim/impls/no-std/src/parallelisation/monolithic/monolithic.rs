use necsim_core::{
    cogs::{
        ActiveLineageSampler, CoalescenceSampler, DispersalSampler, EventSampler, Habitat,
        LineageReference, LocallyCoherentLineageStore, RngCore, SpeciationProbability,
        TurnoverRate,
    },
    simulation::Simulation,
};

use crate::{
    cogs::{
        emigration_exit::never::NeverEmigrationExit,
        immigration_entry::never::NeverImmigrationEntry,
    },
    partitioning::LocalPartition,
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
    E: EventSampler<H, G, R, S, NeverEmigrationExit, D, C, T, N>,
    A: ActiveLineageSampler<H, G, R, S, NeverEmigrationExit, D, C, T, N, E, NeverImmigrationEntry>,
    P: ReporterContext,
    L: LocalPartition<P>,
>(
    simulation: Simulation<
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
    local_partition: &mut L,
) -> (f64, u64) {
    // Ensure that the progress bar starts with the expected target
    local_partition.report_progress_sync(simulation.get_balanced_remaining_work().0);

    let (time, steps, _rng) = simulation.simulate(local_partition.get_reporter());

    local_partition.report_progress_sync(0_u64);

    local_partition.reduce_global_time_steps(time, steps)
}
