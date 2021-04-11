use std::marker::PhantomData;

use necsim_core::{
    cogs::{
        ActiveLineageSampler, BackedUp, Backup, GloballyCoherentLineageStore, Habitat,
        LineageReference, RngCore, SeparableDispersalSampler, SplittableRng,
    },
    lineage::MigratingLineage,
    simulation::{partial::event_sampler::PartialSimulation, Simulation},
};

use necsim_impls_no_std::{
    cogs::{
        coalescence_sampler::conditional::ConditionalCoalescenceSampler,
        emigration_exit::domain::DomainEmigrationExit,
        event_sampler::gillespie::conditional::ConditionalGillespieEventSampler,
        immigration_entry::buffered::BufferedImmigrationEntry,
        speciation_probability::uniform::UniformSpeciationProbability,
        turnover_rate::uniform::UniformTurnoverRate,
    },
    decomposition::Decomposition,
    partitioning::{LocalPartition, MigrationMode},
    reporter::ReporterContext,
};

use necsim_impls_std::cogs::{
    active_lineage_sampler::gillespie::GillespieActiveLineageSampler, rng::pcg::Pcg,
};

use super::reporter::PartitionReporterProxy;

#[allow(clippy::too_many_lines, clippy::too_many_arguments)]
pub fn simulate<
    H: Habitat,
    D: SeparableDispersalSampler<H, Pcg>,
    R: LineageReference<H>,
    S: GloballyCoherentLineageStore<H, R>,
    P: ReporterContext,
    L: LocalPartition<P>,
    C: Decomposition<H>,
>(
    habitat_in: H,
    dispersal_sampler_in: D,
    lineage_store_in: S,
    speciation_probability_per_generation: f64,
    seed: u64,
    local_partition: &mut L,
    decomposition: C,
    independent_time_slice: f64,
) -> (f64, u64) {
    // Create a unique RNG seed for each partition
    let mut rng =
        Pcg::seed_from_u64(seed).split_to_stream(u64::from(local_partition.get_partition_rank()));

    let speciation_probability =
        UniformSpeciationProbability::new(speciation_probability_per_generation);
    let emigration_exit = DomainEmigrationExit::new(decomposition);
    let coalescence_sampler = ConditionalCoalescenceSampler::default();
    let turnover_rate = UniformTurnoverRate::default();
    let event_sampler = ConditionalGillespieEventSampler::default();

    // Pack a PartialSimulation to initialise the GillespieActiveLineageSampler
    let partial_simulation = PartialSimulation {
        habitat: habitat_in,
        speciation_probability,
        dispersal_sampler: dispersal_sampler_in,
        lineage_reference: PhantomData::<R>,
        lineage_store: lineage_store_in,
        emigration_exit,
        coalescence_sampler,
        turnover_rate,
        _rng: PhantomData::<Pcg>,
    };

    let active_lineage_sampler =
        GillespieActiveLineageSampler::new(&partial_simulation, &event_sampler, &mut rng);

    // Unpack the PartialSimulation to create the full Simulation
    let PartialSimulation {
        habitat,
        speciation_probability,
        dispersal_sampler,
        lineage_reference,
        lineage_store,
        emigration_exit,
        coalescence_sampler,
        turnover_rate,
        _rng: _,
    } = partial_simulation;

    let immigration_entry = BufferedImmigrationEntry::default();

    let mut simulation = Simulation::builder()
        .habitat(habitat)
        .rng(rng)
        .speciation_probability(speciation_probability)
        .dispersal_sampler(dispersal_sampler)
        .lineage_reference(lineage_reference)
        .lineage_store(lineage_store)
        .emigration_exit(emigration_exit)
        .coalescence_sampler(coalescence_sampler)
        .turnover_rate(turnover_rate)
        .event_sampler(event_sampler)
        .immigration_entry(immigration_entry)
        .active_lineage_sampler(active_lineage_sampler)
        .build();

    // Ensure that the progress bar starts with the expected target
    local_partition.report_progress_sync(simulation.get_balanced_remaining_work().0);

    let mut global_safe_time = 0.0_f64;
    let mut simulation_backup = simulation.backup();

    let mut last_immigrants: Vec<BackedUp<MigratingLineage>> = Vec::new();
    let mut immigrants: Vec<MigratingLineage> = Vec::new();

    let mut total_steps = 0_u64;

    let mut proxy = PartitionReporterProxy::from(local_partition);

    while proxy
        .local_partition()
        .reduce_vote_continue(simulation.peek_time_of_next_event().is_some())
    {
        loop {
            let (_, new_steps) = simulation.simulate_incremental_until_before(
                global_safe_time + independent_time_slice,
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
                    &mut std::iter::empty(),
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
        global_safe_time += independent_time_slice;
    }

    proxy.local_partition().reduce_global_time_steps(
        simulation.active_lineage_sampler().get_last_event_time(),
        total_steps,
    )
}
