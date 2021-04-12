#![deny(clippy::pedantic)]
#![feature(never_type)]
#![feature(drain_filter)]

#[macro_use]
extern crate serde_derive_state;

use std::collections::VecDeque;

use arguments::{
    AbsoluteDedupCache, DedupCache, IndependentArguments, IsolatedParallelismMode,
    MonolithicParallelismMode, ParallelismMode, RelativeDedupCache,
};
use necsim_core::{
    cogs::{Habitat, RngCore, SpeciationSample},
    lineage::{GlobalLineageReference, Lineage},
    simulation::Simulation,
};

use necsim_impls_no_std::{
    cache::DirectMappedCache as LruCache,
    cogs::{
        active_lineage_sampler::independent::{
            event_time_sampler::exp::ExpEventTimeSampler, IndependentActiveLineageSampler,
        },
        coalescence_sampler::independent::IndependentCoalescenceSampler,
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
        lineage_reference::in_memory::InMemoryLineageReference,
        lineage_store::{
            coherent::locally::classical::ClassicalLineageStore,
            independent::IndependentLineageStore,
        },
        origin_sampler::{
            decomposition::DecompositionOriginSampler, pre_sampler::OriginPreSampler,
        },
        rng::seahash::SeaHash,
    },
    partitioning::LocalPartition,
    reporter::ReporterContext,
};

use necsim_algorithms::{Algorithm, AlgorithmArguments};
use necsim_scenarios::Scenario;
use reporter::IgnoreProgressReporterProxy;

mod arguments;
mod parallelism;
mod reporter;

#[allow(clippy::module_name_repetitions, clippy::empty_enum)]
pub enum IndependentAlgorithm {}

impl AlgorithmArguments for IndependentAlgorithm {
    type Arguments = IndependentArguments;
}

#[allow(clippy::type_complexity)]
impl<
        H: Habitat,
        O: Scenario<
            SeaHash,
            ClassicalLineageStore<H>, // Meaningless
            Habitat = H,
            LineageReference = InMemoryLineageReference, // Meaningless
        >,
    > Algorithm<ClassicalLineageStore<H>, O> for IndependentAlgorithm
{
    type Error = !;
    type LineageReference = GlobalLineageReference;
    type LineageStore = IndependentLineageStore<H>;
    type Rng = SeaHash;

    #[allow(clippy::too_many_lines)]
    fn initialise_and_simulate<
        I: Iterator<Item = u64>,
        R: ReporterContext,
        P: LocalPartition<R>,
    >(
        args: Self::Arguments,
        seed: u64,
        scenario: O,
        pre_sampler: OriginPreSampler<I>,
        local_partition: &mut P,
    ) -> Result<(f64, u64), Self::Error> {
        let decomposition = match args.parallelism_mode {
            ParallelismMode::IsolatedLandscape(IsolatedParallelismMode { partition, .. }) => {
                scenario.decompose(partition.rank(), partition.partitions())
            },
            _ => scenario.decompose(
                local_partition.get_partition_rank(),
                local_partition.get_number_of_partitions(),
            ),
        };

        let lineages: VecDeque<Lineage> = match args.parallelism_mode {
            // Apply no lineage origin partitioning in the `Monolithic` mode
            ParallelismMode::Monolithic(..) => scenario
                .sample_habitat(pre_sampler)
                .map(|indexed_location| Lineage::new(indexed_location, scenario.habitat()))
                .collect(),
            // Apply lineage origin partitioning in the `IsolatedIndividuals` mode
            ParallelismMode::IsolatedIndividuals(IsolatedParallelismMode { partition, .. }) => {
                scenario
                    .sample_habitat(
                        pre_sampler.partition(partition.rank(), partition.partitions().get()),
                    )
                    .map(|indexed_location| Lineage::new(indexed_location, scenario.habitat()))
                    .collect()
            },
            // Apply lineage origin partitioning in the `IsolatedLandscape` mode
            ParallelismMode::IsolatedLandscape(..) => DecompositionOriginSampler::new(
                scenario.sample_habitat(pre_sampler),
                &decomposition,
            )
            .map(|indexed_location| Lineage::new(indexed_location, scenario.habitat()))
            .collect(),
            // Apply lineage origin partitioning in the `Individuals` mode
            ParallelismMode::Individuals => scenario
                .sample_habitat(pre_sampler.partition(
                    local_partition.get_partition_rank(),
                    local_partition.get_number_of_partitions().get(),
                ))
                .map(|indexed_location| Lineage::new(indexed_location, scenario.habitat()))
                .collect(),
            // Apply lineage origin decomposition in the `Landscape` mode
            ParallelismMode::Landscape | ParallelismMode::Probabilistic => {
                DecompositionOriginSampler::new(
                    scenario.sample_habitat(pre_sampler),
                    &decomposition,
                )
                .map(|indexed_location| Lineage::new(indexed_location, scenario.habitat()))
                .collect()
            },
        };

        let (habitat, dispersal_sampler, turnover_rate, speciation_probability) = scenario.build();
        let rng = SeaHash::seed_from_u64(seed);
        let lineage_store = IndependentLineageStore::default();
        let coalescence_sampler = IndependentCoalescenceSampler::default();

        let min_spec_samples: LruCache<SpeciationSample> =
            LruCache::with_capacity(match args.dedup_cache {
                DedupCache::Absolute(AbsoluteDedupCache { capacity }) => capacity.get(),
                DedupCache::Relative(RelativeDedupCache { factor }) => {
                    #[allow(
                        clippy::cast_precision_loss,
                        clippy::cast_sign_loss,
                        clippy::cast_possible_truncation
                    )]
                    let capacity = ((lineages.len() as f64) * factor.get()) as usize;

                    capacity
                },
                DedupCache::None => 0_usize,
            });

        let mut proxy = IgnoreProgressReporterProxy::from(local_partition);

        match args.parallelism_mode {
            ParallelismMode::Monolithic(MonolithicParallelismMode { event_slice })
            | ParallelismMode::IsolatedIndividuals(IsolatedParallelismMode {
                event_slice, ..
            })
            | ParallelismMode::IsolatedLandscape(IsolatedParallelismMode { event_slice, .. }) => {
                let emigration_exit = NeverEmigrationExit::default();
                let event_sampler = IndependentEventSampler::default();
                let immigration_entry = NeverImmigrationEntry::default();
                let active_lineage_sampler = IndependentActiveLineageSampler::empty(
                    ExpEventTimeSampler::new(args.delta_t.get()),
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

                Ok(parallelism::monolithic::simulate(
                    simulation,
                    lineages,
                    min_spec_samples,
                    args.step_slice,
                    event_slice,
                    &mut proxy,
                ))
            },
            ParallelismMode::Individuals => {
                let emigration_exit = NeverEmigrationExit::default();
                let event_sampler = IndependentEventSampler::default();
                let immigration_entry = NeverImmigrationEntry::default();
                let active_lineage_sampler = IndependentActiveLineageSampler::empty(
                    ExpEventTimeSampler::new(args.delta_t.get()),
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

                Ok(parallelism::individuals::simulate(
                    simulation,
                    lineages,
                    min_spec_samples,
                    args.step_slice,
                    &mut proxy,
                ))
            },
            ParallelismMode::Landscape => {
                let emigration_exit = IndependentEmigrationExit::new(
                    decomposition,
                    AlwaysEmigrationChoice::default(),
                );
                let event_sampler = IndependentEventSampler::default();
                let immigration_entry = NeverImmigrationEntry::default();
                let active_lineage_sampler = IndependentActiveLineageSampler::empty(
                    ExpEventTimeSampler::new(args.delta_t.get()),
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

                Ok(parallelism::landscape::simulate(
                    simulation,
                    lineages,
                    min_spec_samples,
                    args.step_slice,
                    &mut proxy,
                ))
            },
            ParallelismMode::Probabilistic => {
                let emigration_exit = IndependentEmigrationExit::new(
                    decomposition,
                    ProbabilisticEmigrationChoice::default(),
                );
                let event_sampler = IndependentEventSampler::default();
                let immigration_entry = NeverImmigrationEntry::default();
                let active_lineage_sampler = IndependentActiveLineageSampler::empty(
                    ExpEventTimeSampler::new(args.delta_t.get()),
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

                Ok(parallelism::landscape::simulate(
                    simulation,
                    lineages,
                    min_spec_samples,
                    args.step_slice,
                    &mut proxy,
                ))
            },
        }
    }
}
