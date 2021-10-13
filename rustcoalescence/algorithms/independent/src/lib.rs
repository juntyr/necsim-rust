#![deny(clippy::pedantic)]
#![feature(never_type)]

#[macro_use]
extern crate serde_derive_state;

use std::marker::PhantomData;

use arguments::{
    IndependentArguments, IsolatedParallelismMode, MonolithicParallelismMode, ParallelismMode,
    ProbabilisticParallelismMode,
};
use necsim_core::{
    cogs::SeedableRng,
    lineage::{GlobalLineageReference, Lineage},
    reporter::Reporter,
    simulation::SimulationBuilder,
};
use necsim_core_bond::NonNegativeF64;
use necsim_core_f64::IntrinsicsF64Core;

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
};
use necsim_partitioning_core::LocalPartition;

mod arguments;

use rustcoalescence_algorithms::{Algorithm, AlgorithmArguments};
use rustcoalescence_scenarios::Scenario;

#[allow(clippy::module_name_repetitions, clippy::empty_enum)]
pub enum IndependentAlgorithm {}

impl AlgorithmArguments for IndependentAlgorithm {
    type Arguments = IndependentArguments;
}

#[allow(clippy::type_complexity)]
impl<
        O: Scenario<IntrinsicsF64Core, WyHash<IntrinsicsF64Core>>,
        R: Reporter,
        P: LocalPartition<R>,
    > Algorithm<O, R, P> for IndependentAlgorithm
{
    type Error = !;
    type F64Core = IntrinsicsF64Core;
    type LineageReference = GlobalLineageReference;
    type LineageStore = IndependentLineageStore<IntrinsicsF64Core, O::Habitat>;
    type Rng = WyHash<IntrinsicsF64Core>;

    #[allow(clippy::too_many_lines)]
    fn initialise_and_simulate<I: Iterator<Item = u64>>(
        args: Self::Arguments,
        seed: u64,
        scenario: O,
        pre_sampler: OriginPreSampler<Self::F64Core, I>,
        local_partition: &mut P,
    ) -> Result<(NonNegativeF64, u64), Self::Error> {
        match args.parallelism_mode {
            ParallelismMode::Monolithic(MonolithicParallelismMode { event_slice })
            | ParallelismMode::IsolatedIndividuals(IsolatedParallelismMode {
                event_slice, ..
            })
            | ParallelismMode::IsolatedLandscape(IsolatedParallelismMode { event_slice, .. }) => {
                let lineages: Vec<Lineage> = match args.parallelism_mode {
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
                    _ => unsafe { std::hint::unreachable_unchecked() },
                };

                let (habitat, dispersal_sampler, turnover_rate, speciation_probability) =
                    scenario.build::<InMemoryAliasDispersalSampler<
                        Self::F64Core,
                        O::Habitat,
                        WyHash<Self::F64Core>,
                    >>();
                let rng = WyHash::seed_from_u64(seed);
                let lineage_store = IndependentLineageStore::default();
                let coalescence_sampler = IndependentCoalescenceSampler::default();

                let emigration_exit = NeverEmigrationExit::default();
                let event_sampler = IndependentEventSampler::default();
                let immigration_entry = NeverImmigrationEntry::default();
                let active_lineage_sampler = IndependentActiveLineageSampler::empty(
                    PoissonEventTimeSampler::new(args.delta_t),
                );

                let simulation = SimulationBuilder {
                    f64_core: PhantomData::<Self::F64Core>,
                    habitat,
                    lineage_reference: PhantomData::<GlobalLineageReference>,
                    lineage_store,
                    dispersal_sampler,
                    coalescence_sampler,
                    turnover_rate,
                    speciation_probability,
                    emigration_exit,
                    event_sampler,
                    active_lineage_sampler,
                    rng,
                    immigration_entry,
                }
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
                let lineages: Vec<Lineage> = scenario
                    .sample_habitat(pre_sampler.partition(
                        local_partition.get_partition_rank(),
                        local_partition.get_number_of_partitions().get(),
                    ))
                    .map(|indexed_location| Lineage::new(indexed_location, scenario.habitat()))
                    .collect();

                let (habitat, dispersal_sampler, turnover_rate, speciation_probability) =
                    scenario.build::<InMemoryAliasDispersalSampler<
                        Self::F64Core,
                        O::Habitat,
                        WyHash<Self::F64Core>,
                    >>();
                let rng = WyHash::seed_from_u64(seed);
                let lineage_store = IndependentLineageStore::default();
                let coalescence_sampler = IndependentCoalescenceSampler::default();
                let emigration_exit = NeverEmigrationExit::default();
                let event_sampler = IndependentEventSampler::default();
                let immigration_entry = NeverImmigrationEntry::default();
                let active_lineage_sampler = IndependentActiveLineageSampler::empty(
                    PoissonEventTimeSampler::new(args.delta_t),
                );

                let simulation = SimulationBuilder {
                    f64_core: PhantomData::<Self::F64Core>,
                    habitat,
                    lineage_reference: PhantomData::<GlobalLineageReference>,
                    lineage_store,
                    dispersal_sampler,
                    coalescence_sampler,
                    turnover_rate,
                    speciation_probability,
                    emigration_exit,
                    event_sampler,
                    active_lineage_sampler,
                    rng,
                    immigration_entry,
                }
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
                let lineages: Vec<Lineage> = DecompositionOriginSampler::new(
                    scenario.sample_habitat(pre_sampler),
                    &decomposition,
                )
                .map(|indexed_location| Lineage::new(indexed_location, scenario.habitat()))
                .collect();

                let (habitat, dispersal_sampler, turnover_rate, speciation_probability) =
                    scenario.build::<InMemoryAliasDispersalSampler<
                        Self::F64Core,
                        O::Habitat,
                        WyHash<Self::F64Core>,
                    >>();
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
                    PoissonEventTimeSampler::new(args.delta_t),
                );

                let simulation = SimulationBuilder {
                    f64_core: PhantomData::<Self::F64Core>,
                    habitat,
                    lineage_reference: PhantomData::<GlobalLineageReference>,
                    lineage_store,
                    dispersal_sampler,
                    coalescence_sampler,
                    turnover_rate,
                    speciation_probability,
                    emigration_exit,
                    event_sampler,
                    active_lineage_sampler,
                    rng,
                    immigration_entry,
                }
                .build();

                Ok(parallelisation::independent::landscape::simulate(
                    simulation,
                    lineages,
                    args.dedup_cache,
                    args.step_slice,
                    local_partition,
                ))
            },
            ParallelismMode::Probabilistic(ProbabilisticParallelismMode {
                communication_probability,
            }) => {
                let decomposition = O::decompose(
                    scenario.habitat(),
                    local_partition.get_partition_rank(),
                    local_partition.get_number_of_partitions(),
                );
                let lineages: Vec<Lineage> = DecompositionOriginSampler::new(
                    scenario.sample_habitat(pre_sampler),
                    &decomposition,
                )
                .map(|indexed_location| Lineage::new(indexed_location, scenario.habitat()))
                .collect();

                let (habitat, dispersal_sampler, turnover_rate, speciation_probability) =
                    scenario.build::<InMemoryAliasDispersalSampler<
                        Self::F64Core,
                        O::Habitat,
                        WyHash<Self::F64Core>,
                    >>();
                let rng = WyHash::seed_from_u64(seed);
                let lineage_store = IndependentLineageStore::default();
                let coalescence_sampler = IndependentCoalescenceSampler::default();
                let emigration_exit = IndependentEmigrationExit::new(
                    decomposition,
                    ProbabilisticEmigrationChoice::new(communication_probability),
                );
                let event_sampler = IndependentEventSampler::default();
                let immigration_entry = NeverImmigrationEntry::default();
                let active_lineage_sampler = IndependentActiveLineageSampler::empty(
                    PoissonEventTimeSampler::new(args.delta_t),
                );

                let simulation = SimulationBuilder {
                    f64_core: PhantomData::<Self::F64Core>,
                    habitat,
                    lineage_reference: PhantomData::<GlobalLineageReference>,
                    lineage_store,
                    dispersal_sampler,
                    coalescence_sampler,
                    turnover_rate,
                    speciation_probability,
                    emigration_exit,
                    event_sampler,
                    active_lineage_sampler,
                    rng,
                    immigration_entry,
                }
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
