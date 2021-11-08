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
    lineage::{GlobalLineageReference, Lineage},
    reporter::Reporter,
    simulation::SimulationBuilder,
};
use necsim_core_bond::NonNegativeF64;
use necsim_core_maths::IntrinsicsMathsCore;

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
    parallelisation::{self, Status},
};
use necsim_partitioning_core::LocalPartition;

mod arguments;

use rustcoalescence_algorithms::{Algorithm, AlgorithmArguments, AlgorithmResult};
use rustcoalescence_scenarios::Scenario;

#[allow(clippy::module_name_repetitions, clippy::empty_enum)]
pub enum IndependentAlgorithm {}

impl AlgorithmArguments for IndependentAlgorithm {
    type Arguments = IndependentArguments;
}

#[allow(clippy::type_complexity)]
impl<
        O: Scenario<IntrinsicsMathsCore, WyHash<IntrinsicsMathsCore>>,
        R: Reporter,
        P: LocalPartition<R>,
    > Algorithm<O, R, P> for IndependentAlgorithm
{
    type Error = !;
    type LineageReference = GlobalLineageReference;
    type LineageStore = IndependentLineageStore<IntrinsicsMathsCore, O::Habitat>;
    type MathsCore = IntrinsicsMathsCore;
    type Rng = WyHash<IntrinsicsMathsCore>;

    #[allow(clippy::too_many_lines)]
    fn initialise_and_simulate<I: Iterator<Item = u64>>(
        args: Self::Arguments,
        rng: Self::Rng,
        scenario: O,
        pre_sampler: OriginPreSampler<Self::MathsCore, I>,
        pause_before: Option<NonNegativeF64>,
        local_partition: &mut P,
    ) -> Result<AlgorithmResult<Self::MathsCore, Self::Rng>, Self::Error> {
        match args.parallelism_mode {
            ParallelismMode::Monolithic(MonolithicParallelismMode { event_slice })
            | ParallelismMode::IsolatedIndividuals(IsolatedParallelismMode {
                event_slice, ..
            })
            | ParallelismMode::IsolatedLandscape(IsolatedParallelismMode { event_slice, .. }) => {
                let lineages: Vec<Lineage> = match args.parallelism_mode {
                    // Apply no lineage origin partitioning in the `Monolithic` mode
                    ParallelismMode::Monolithic(..) => {
                        scenario.sample_habitat(pre_sampler).collect()
                    },
                    // Apply lineage origin partitioning in the `IsolatedIndividuals` mode
                    ParallelismMode::IsolatedIndividuals(IsolatedParallelismMode {
                        partition,
                        ..
                    }) => scenario
                        .sample_habitat(pre_sampler.partition(partition))
                        .collect(),
                    // Apply lineage origin partitioning in the `IsolatedLandscape` mode
                    ParallelismMode::IsolatedLandscape(IsolatedParallelismMode {
                        partition,
                        ..
                    }) => DecompositionOriginSampler::new(
                        scenario.sample_habitat(pre_sampler),
                        &O::decompose(scenario.habitat(), partition),
                    )
                    .collect(),
                    _ => unsafe { std::hint::unreachable_unchecked() },
                };

                let (habitat, dispersal_sampler, turnover_rate, speciation_probability) =
                    scenario.build::<InMemoryAliasDispersalSampler<
                        Self::MathsCore,
                        O::Habitat,
                        WyHash<Self::MathsCore>,
                    >>();
                let lineage_store = IndependentLineageStore::default();
                let coalescence_sampler = IndependentCoalescenceSampler::default();

                let emigration_exit = NeverEmigrationExit::default();
                let event_sampler = IndependentEventSampler::default();
                let immigration_entry = NeverImmigrationEntry::default();
                let active_lineage_sampler = IndependentActiveLineageSampler::empty(
                    PoissonEventTimeSampler::new(args.delta_t),
                );

                let mut simulation = SimulationBuilder {
                    maths: PhantomData::<Self::MathsCore>,
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

                let (status, time, steps, lineages) =
                    parallelisation::independent::monolithic::simulate(
                        &mut simulation,
                        lineages,
                        args.dedup_cache,
                        args.step_slice,
                        event_slice,
                        pause_before,
                        local_partition,
                    );

                match status {
                    Status::Done => Ok(AlgorithmResult::Done { time, steps }),
                    Status::Paused => Ok(AlgorithmResult::Paused {
                        time,
                        steps,
                        lineages: lineages.into_iter().collect(),
                        rng: simulation.rng_mut().clone(),
                        marker: PhantomData,
                    }),
                }
            },
            ParallelismMode::Individuals => {
                let lineages: Vec<Lineage> = scenario
                    .sample_habitat(pre_sampler.partition(local_partition.get_partition()))
                    .collect();

                let (habitat, dispersal_sampler, turnover_rate, speciation_probability) =
                    scenario.build::<InMemoryAliasDispersalSampler<
                        Self::MathsCore,
                        O::Habitat,
                        WyHash<Self::MathsCore>,
                    >>();
                let lineage_store = IndependentLineageStore::default();
                let coalescence_sampler = IndependentCoalescenceSampler::default();
                let emigration_exit = NeverEmigrationExit::default();
                let event_sampler = IndependentEventSampler::default();
                let immigration_entry = NeverImmigrationEntry::default();
                let active_lineage_sampler = IndependentActiveLineageSampler::empty(
                    PoissonEventTimeSampler::new(args.delta_t),
                );

                let mut simulation = SimulationBuilder {
                    maths: PhantomData::<Self::MathsCore>,
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

                let (_status, time, steps, _lineages) =
                    parallelisation::independent::individuals::simulate(
                        &mut simulation,
                        lineages,
                        args.dedup_cache,
                        args.step_slice,
                        local_partition,
                    );

                // TODO: Adapt for parallel pausing
                Ok(AlgorithmResult::Done { time, steps })
            },
            ParallelismMode::Landscape => {
                let decomposition =
                    O::decompose(scenario.habitat(), local_partition.get_partition());
                let lineages: Vec<Lineage> = DecompositionOriginSampler::new(
                    scenario.sample_habitat(pre_sampler),
                    &decomposition,
                )
                .collect();

                let (habitat, dispersal_sampler, turnover_rate, speciation_probability) =
                    scenario.build::<InMemoryAliasDispersalSampler<
                        Self::MathsCore,
                        O::Habitat,
                        WyHash<Self::MathsCore>,
                    >>();
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

                let mut simulation = SimulationBuilder {
                    maths: PhantomData::<Self::MathsCore>,
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

                let (_status, time, steps, _lineages) =
                    parallelisation::independent::landscape::simulate(
                        &mut simulation,
                        lineages,
                        args.dedup_cache,
                        args.step_slice,
                        local_partition,
                    );

                // TODO: Adapt for parallel pausing
                Ok(AlgorithmResult::Done { time, steps })
            },
            ParallelismMode::Probabilistic(ProbabilisticParallelismMode {
                communication_probability,
            }) => {
                let decomposition =
                    O::decompose(scenario.habitat(), local_partition.get_partition());
                let lineages: Vec<Lineage> = DecompositionOriginSampler::new(
                    scenario.sample_habitat(pre_sampler),
                    &decomposition,
                )
                .collect();

                let (habitat, dispersal_sampler, turnover_rate, speciation_probability) =
                    scenario.build::<InMemoryAliasDispersalSampler<
                        Self::MathsCore,
                        O::Habitat,
                        WyHash<Self::MathsCore>,
                    >>();
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

                let mut simulation = SimulationBuilder {
                    maths: PhantomData::<Self::MathsCore>,
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

                let (_status, time, steps, _lineages) =
                    parallelisation::independent::landscape::simulate(
                        &mut simulation,
                        lineages,
                        args.dedup_cache,
                        args.step_slice,
                        local_partition,
                    );

                // TODO: Adapt for parallel pausing
                Ok(AlgorithmResult::Done { time, steps })
            },
        }
    }
}
