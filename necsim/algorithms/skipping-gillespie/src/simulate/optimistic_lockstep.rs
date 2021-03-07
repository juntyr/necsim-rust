use std::marker::PhantomData;

use necsim_core::{
    cogs::{
        ActiveLineageSampler, Backup, CoherentLineageStore, Habitat, LineageReference, RngCore,
        SeparableDispersalSampler,
    },
    reporter::{NullReporter, Reporter},
    simulation::{partial::event_sampler::PartialSimulation, Simulation},
};

use necsim_impls_no_std::{
    cogs::{
        coalescence_sampler::conditional::ConditionalCoalescenceSampler,
        emigration_exit::domain::DomainEmigrationExit,
        event_sampler::gillespie::conditional::ConditionalGillespieEventSampler,
        immigration_entry::buffered::BufferedImmigrationEntry,
        speciation_probability::uniform::UniformSpeciationProbability,
    },
    decomposition::Decomposition,
    partitioning::{LocalPartition, MigrationMode},
};

use necsim_impls_std::cogs::{
    active_lineage_sampler::gillespie::GillespieActiveLineageSampler, rng::std::StdRng,
};

use necsim_impls_no_std::reporter::ReporterContext;

#[allow(clippy::too_many_lines)]
pub fn simulate<
    H: Habitat,
    D: SeparableDispersalSampler<H, StdRng>,
    R: LineageReference<H>,
    S: CoherentLineageStore<H, R>,
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
) -> (f64, u64) {
    // Create a unique RNG seed for each partition
    let mut rng = StdRng::seed_from_u64(seed);
    for _ in 0..local_partition.get_partition_rank() {
        let _ = rng.sample_u64();
    }
    let mut rng = StdRng::seed_from_u64(rng.sample_u64());

    let speciation_probability =
        UniformSpeciationProbability::new(speciation_probability_per_generation);
    let emigration_exit = DomainEmigrationExit::new(decomposition);
    let coalescence_sampler = ConditionalCoalescenceSampler::default();
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
        rng: PhantomData::<StdRng>,
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
        rng: _,
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
        .event_sampler(event_sampler)
        .immigration_entry(immigration_entry)
        .active_lineage_sampler(active_lineage_sampler)
        .build();

    // TODO: This is hacky but ensures that the progress bar has the expected target
    if !local_partition.is_root() {
        local_partition
            .get_reporter()
            .report_progress(simulation.active_lineage_sampler().number_active_lineages() as u64);
    }
    local_partition.reduce_vote_continue(true);
    if local_partition.is_root() {
        local_partition
            .get_reporter()
            .report_progress(simulation.active_lineage_sampler().number_active_lineages() as u64);
    }

    let mut simulation_backup = simulation.backup();

    let mut total_steps = 0_u64;

    while local_partition.reduce_vote_continue(simulation.peek_time_of_next_event().is_some()) {
        // Get the next local emigration event time or +inf
        //  (we already know at least one partition has some next event time)
        let next_local_emigration_time = loop {
            let (_, new_steps) = simulation.simulate_incremental_for(1, &mut NullReporter);

            total_steps += new_steps;

            if new_steps == 0 {
                break None;
            }

            if let Some((_, first_emigration)) = simulation.emigration_exit_mut().min() {
                break Some(first_emigration.event_time);
            }
        }
        .unwrap_or(f64::INFINITY);

        // Roll back the simulation to the latest safe point
        simulation = simulation_backup.resume();

        match local_partition.reduce_vote_min_time(next_local_emigration_time) {
            // The partition with the next emigration event gets to simulate until and inclusive
            //  that event
            Ok(next_global_time) => {
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
            },
            // All other partitions get to simulate until just before this next migration event
            Err(next_global_time) => {
                let (_, new_steps) = simulation.simulate_incremental_until_before(
                    next_global_time,
                    local_partition.get_reporter(),
                );

                total_steps += new_steps;
            },
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

        // Advance the simulation backup to this new safe point
        simulation_backup = simulation.backup();
    }

    local_partition.reduce_global_time_steps(
        simulation.active_lineage_sampler().get_time_of_last_event(),
        total_steps,
    )
}
