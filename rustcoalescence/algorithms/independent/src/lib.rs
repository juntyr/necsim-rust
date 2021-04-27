#![deny(clippy::pedantic)]
#![feature(never_type)]
#![feature(drain_filter)]

#[macro_use]
extern crate serde_derive_state;

use std::{collections::VecDeque, hint::unreachable_unchecked};

use arguments::{
    IndependentArguments, IsolatedParallelismMode, MonolithicParallelismMode, ParallelismMode,
};
use necsim_core::{
    cogs::RngCore,
    lineage::{GlobalLineageReference, Lineage},
    reporter::Reporter,
    simulation::Simulation,
};

use necsim_impls_no_std::{
    cogs::{
        active_lineage_sampler::independent::{
            event_time_sampler::poisson::PoissonEventTimeSampler, IndependentActiveLineageSampler,
        },
        coalescence_sampler::independent::IndependentCoalescenceSampler,
        dispersal_sampler::in_memory::alias::InMemoryAliasDispersalSampler,
        emigration_exit::{
            independent::{
                choice::{
                    always::AlwaysEmigrationChoice, probabilistic::ProbabilisticEmigrationChoice,
                },
                IndependentEmigrationExit,
            },
            never::NeverEmigrationExit,
        },
        event_sampler::independent::IndependentEventSampler,
        immigration_entry::never::NeverImmigrationEntry,
        lineage_store::independent::IndependentLineageStore,
        origin_sampler::{
            decomposition::DecompositionOriginSampler, pre_sampler::OriginPreSampler,
        },
        rng::wyhash::WyHash,
    },
    parallelisation,
    partitioning::LocalPartition,
};

mod arguments;

use rustcoalescence_algorithms::{Algorithm, AlgorithmArguments};
use rustcoalescence_scenarios::Scenario;

#[allow(clippy::module_name_repetitions, clippy::empty_enum)]
pub enum IndependentAlgorithm {}

impl AlgorithmArguments for IndependentAlgorithm {
    type Arguments = IndependentArguments;
}

#[allow(clippy::type_complexity)]
impl<O: Scenario<WyHash>> Algorithm<O> for IndependentAlgorithm {
    type Error = !;
    type LineageReference = GlobalLineageReference;
    type LineageStore = IndependentLineageStore<O::Habitat>;
    type Rng = WyHash;

    #[allow(clippy::too_many_lines)]
    fn initialise_and_simulate<I: Iterator<Item = u64>, R: Reporter, P: LocalPartition<R>>(
        args: Self::Arguments,
        seed: u64,
        scenario: O,
        pre_sampler: OriginPreSampler<I>,
        local_partition: &mut P,
    ) -> Result<(f64, u64), Self::Error> {
        match args.parallelism_mode {
            ParallelismMode::Monolithic(MonolithicParallelismMode { event_slice })
            | ParallelismMode::IsolatedIndividuals(IsolatedParallelismMode {
                event_slice, ..
            })
            | ParallelismMode::IsolatedLandscape(IsolatedParallelismMode { event_slice, .. }) => {
                let lineages: VecDeque<Lineage> = match args.parallelism_mode {
                    // Apply no lineage origin partitioning in the `Monolithic` mode
                    ParallelismMode::Monolithic(..) => scenario
                        .sample_habitat(pre_sampler)
                        .map(|indexed_location| Lineage::new(indexed_location, scenario.habitat()))
                        .collect(),
                    // Apply lineage origin partitioning in the `IsolatedIndividuals` mode
                    ParallelismMode::IsolatedIndividuals(IsolatedParallelismMode {
                        partition,
                        ..
                    }) => scenario
                        .sample_habitat(
                            pre_sampler.partition(partition.rank(), partition.partitions().get()),
                        )
                        .map(|indexed_location| Lineage::new(indexed_location, scenario.habitat()))
                        .collect(),
                    // Apply lineage origin partitioning in the `IsolatedLandscape` mode
                    ParallelismMode::IsolatedLandscape(IsolatedParallelismMode {
                        partition,
                        ..
                    }) => DecompositionOriginSampler::new(
                        scenario.sample_habitat(pre_sampler),
                        &O::decompose(scenario.habitat(), partition.rank(), partition.partitions()),
                    )
                    .map(|indexed_location| Lineage::new(indexed_location, scenario.habitat()))
                    .collect(),
                    _ => unsafe { unreachable_unchecked() },
                };

                let (habitat, dispersal_sampler, turnover_rate, speciation_probability) =
                    scenario.build::<InMemoryAliasDispersalSampler<O::Habitat, WyHash>>();
                let rng = WyHash::seed_from_u64(seed);
                let lineage_store = IndependentLineageStore::default();
                let coalescence_sampler = IndependentCoalescenceSampler::default();

                let emigration_exit = NeverEmigrationExit::default();
                let event_sampler = IndependentEventSampler::default();
                let immigration_entry = NeverImmigrationEntry::default();
                let active_lineage_sampler = IndependentActiveLineageSampler::empty(
                    PoissonEventTimeSampler::new(args.delta_t.get()),
                );

                let simulation = Simulation::builder()
                    .habitat(habitat)
                    .rng(rng)
                    .speciation_probability(speciation_probability)
                    .dispersal_sampler(dispersal_sampler)
                    .lineage_reference(std::marker::PhantomData::<GlobalLineageReference>)
                    .lineage_store(lineage_store)
                    .emigration_exit(emigration_exit)
                    .coalescence_sampler(coalescence_sampler)
                    .turnover_rate(turnover_rate)
                    .event_sampler(event_sampler)
                    .immigration_entry(immigration_entry)
                    .active_lineage_sampler(active_lineage_sampler)
                    .build();

                Ok(parallelisation::independent::monolithic::simulate(
                    simulation,
                    lineages,
                    args.dedup_cache,
                    args.step_slice,
                    event_slice,
                    local_partition,
                ))
            },
            ParallelismMode::Individuals => {
                let lineages: VecDeque<Lineage> = scenario
                    .sample_habitat(pre_sampler.partition(
                        local_partition.get_partition_rank(),
                        local_partition.get_number_of_partitions().get(),
                    ))
                    .map(|indexed_location| Lineage::new(indexed_location, scenario.habitat()))
                    .collect();

                let (habitat, dispersal_sampler, turnover_rate, speciation_probability) =
                    scenario.build::<InMemoryAliasDispersalSampler<O::Habitat, WyHash>>();
                let rng = WyHash::seed_from_u64(seed);
                let lineage_store = IndependentLineageStore::default();
                let coalescence_sampler = IndependentCoalescenceSampler::default();
                let emigration_exit = NeverEmigrationExit::default();
                let event_sampler = IndependentEventSampler::default();
                let immigration_entry = NeverImmigrationEntry::default();
                let active_lineage_sampler = IndependentActiveLineageSampler::empty(
                    PoissonEventTimeSampler::new(args.delta_t.get()),
                );

                let simulation = Simulation::builder()
                    .habitat(habitat)
                    .rng(rng)
                    .speciation_probability(speciation_probability)
                    .dispersal_sampler(dispersal_sampler)
                    .lineage_reference(std::marker::PhantomData::<GlobalLineageReference>)
                    .lineage_store(lineage_store)
                    .emigration_exit(emigration_exit)
                    .coalescence_sampler(coalescence_sampler)
                    .turnover_rate(turnover_rate)
                    .event_sampler(event_sampler)
                    .immigration_entry(immigration_entry)
                    .active_lineage_sampler(active_lineage_sampler)
                    .build();

                Ok(parallelisation::independent::individuals::simulate(
                    simulation,
                    lineages,
                    args.dedup_cache,
                    args.step_slice,
                    local_partition,
                ))
            },
            ParallelismMode::Landscape => {
                let decomposition = O::decompose(
                    scenario.habitat(),
                    local_partition.get_partition_rank(),
                    local_partition.get_number_of_partitions(),
                );
                let lineages: VecDeque<Lineage> = DecompositionOriginSampler::new(
                    scenario.sample_habitat(pre_sampler),
                    &decomposition,
                )
                .map(|indexed_location| Lineage::new(indexed_location, scenario.habitat()))
                .collect();

                let (habitat, dispersal_sampler, turnover_rate, speciation_probability) =
                    scenario.build::<InMemoryAliasDispersalSampler<O::Habitat, WyHash>>();
                let rng = WyHash::seed_from_u64(seed);
                let lineage_store = IndependentLineageStore::default();
                let coalescence_sampler = IndependentCoalescenceSampler::default();
                let emigration_exit = IndependentEmigrationExit::new(
                    decomposition,
                    AlwaysEmigrationChoice::default(),
                );
                let event_sampler = IndependentEventSampler::default();
                let immigration_entry = NeverImmigrationEntry::default();
                let active_lineage_sampler = IndependentActiveLineageSampler::empty(
                    PoissonEventTimeSampler::new(args.delta_t.get()),
                );

                let simulation = Simulation::builder()
                    .habitat(habitat)
                    .rng(rng)
                    .speciation_probability(speciation_probability)
                    .dispersal_sampler(dispersal_sampler)
                    .lineage_reference(std::marker::PhantomData::<GlobalLineageReference>)
                    .lineage_store(lineage_store)
                    .emigration_exit(emigration_exit)
                    .coalescence_sampler(coalescence_sampler)
                    .turnover_rate(turnover_rate)
                    .event_sampler(event_sampler)
                    .immigration_entry(immigration_entry)
                    .active_lineage_sampler(active_lineage_sampler)
                    .build();

                Ok(parallelisation::independent::landscape::simulate(
                    simulation,
                    lineages,
                    args.dedup_cache,
                    args.step_slice,
                    local_partition,
                ))
            },
            ParallelismMode::Probabilistic => {
                let decomposition = O::decompose(
                    scenario.habitat(),
                    local_partition.get_partition_rank(),
                    local_partition.get_number_of_partitions(),
                );
                let lineages: VecDeque<Lineage> = DecompositionOriginSampler::new(
                    scenario.sample_habitat(pre_sampler),
                    &decomposition,
                )
                .map(|indexed_location| Lineage::new(indexed_location, scenario.habitat()))
                .collect();

                let (habitat, dispersal_sampler, turnover_rate, speciation_probability) =
                    scenario.build::<InMemoryAliasDispersalSampler<O::Habitat, WyHash>>();
                let rng = WyHash::seed_from_u64(seed);
                let lineage_store = IndependentLineageStore::default();
                let coalescence_sampler = IndependentCoalescenceSampler::default();
                let emigration_exit = IndependentEmigrationExit::new(
                    decomposition,
                    ProbabilisticEmigrationChoice::default(),
                );
                let event_sampler = IndependentEventSampler::default();
                let immigration_entry = NeverImmigrationEntry::default();
                let active_lineage_sampler = IndependentActiveLineageSampler::empty(
                    PoissonEventTimeSampler::new(args.delta_t.get()),
                );

                let simulation = Simulation::builder()
                    .habitat(habitat)
                    .rng(rng)
                    .speciation_probability(speciation_probability)
                    .dispersal_sampler(dispersal_sampler)
                    .lineage_reference(std::marker::PhantomData::<GlobalLineageReference>)
                    .lineage_store(lineage_store)
                    .emigration_exit(emigration_exit)
                    .coalescence_sampler(coalescence_sampler)
                    .turnover_rate(turnover_rate)
                    .event_sampler(event_sampler)
                    .immigration_entry(immigration_entry)
                    .active_lineage_sampler(active_lineage_sampler)
                    .build();

                Ok(parallelisation::independent::landscape::simulate(
                    simulation,
                    lineages,
                    args.dedup_cache,
                    args.step_slice,
                    local_partition,
                ))
            },
        }
    }
}
