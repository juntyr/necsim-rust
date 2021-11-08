use std::{hint::unreachable_unchecked, marker::PhantomData};

use necsim_core::{
    cogs::{ActiveLineageSampler, LineageStore, LocallyCoherentLineageStore, SplittableRng},
    reporter::Reporter,
    simulation::SimulationBuilder,
};
use necsim_core_bond::NonNegativeF64;
use necsim_core_maths::IntrinsicsMathsCore;

use necsim_impls_no_std::{
    cogs::{
        active_lineage_sampler::{
            alias::individual::IndividualAliasActiveLineageSampler,
            classical::ClassicalActiveLineageSampler,
        },
        coalescence_sampler::unconditional::UnconditionalCoalescenceSampler,
        dispersal_sampler::in_memory::alias::InMemoryAliasDispersalSampler,
        emigration_exit::{domain::DomainEmigrationExit, never::NeverEmigrationExit},
        event_sampler::unconditional::UnconditionalEventSampler,
        immigration_entry::{buffered::BufferedImmigrationEntry, never::NeverImmigrationEntry},
        lineage_reference::in_memory::InMemoryLineageReference,
        lineage_store::coherent::locally::classical::ClassicalLineageStore,
        origin_sampler::{
            decomposition::DecompositionOriginSampler, pre_sampler::OriginPreSampler,
        },
        turnover_rate::uniform::UniformTurnoverRate,
    },
    parallelisation::{self, Status},
};
use necsim_impls_std::cogs::rng::pcg::Pcg;
use necsim_partitioning_core::LocalPartition;

use rustcoalescence_algorithms::{Algorithm, AlgorithmArguments, AlgorithmResult};
use rustcoalescence_scenarios::Scenario;

use crate::arguments::{
    AveragingParallelismMode, MonolithicArguments, OptimisticParallelismMode, ParallelismMode,
};

#[allow(clippy::module_name_repetitions, clippy::empty_enum)]
pub enum ClassicalAlgorithm {}

impl AlgorithmArguments for ClassicalAlgorithm {
    type Arguments = MonolithicArguments;
}

// Default 'Gillespie' implementation for any turnover sampler
#[allow(clippy::type_complexity)]
impl<
        O: Scenario<
            IntrinsicsMathsCore,
            Pcg<IntrinsicsMathsCore>,
            LineageReference = InMemoryLineageReference,
        >,
        R: Reporter,
        P: LocalPartition<R>,
    > Algorithm<O, R, P> for ClassicalAlgorithm
where
    O::LineageStore<ClassicalLineageStore<IntrinsicsMathsCore, O::Habitat>>:
        LocallyCoherentLineageStore<IntrinsicsMathsCore, O::Habitat, InMemoryLineageReference>,
{
    type Error = !;
    type LineageReference = InMemoryLineageReference;
    type LineageStore = O::LineageStore<ClassicalLineageStore<Self::MathsCore, O::Habitat>>;
    type MathsCore = IntrinsicsMathsCore;
    type Rng = Pcg<Self::MathsCore>;

    #[allow(clippy::shadow_unrelated, clippy::too_many_lines)]
    default fn initialise_and_simulate<I: Iterator<Item = u64>>(
        args: Self::Arguments,
        rng: Self::Rng,
        scenario: O,
        pre_sampler: OriginPreSampler<Self::MathsCore, I>,
        pause_before: Option<NonNegativeF64>,
        local_partition: &mut P,
    ) -> Result<AlgorithmResult<Self::MathsCore, Self::Rng>, Self::Error> {
        match args.parallelism_mode {
            ParallelismMode::Monolithic => {
                let lineage_store =
                    Self::LineageStore::from_origin_sampler(scenario.sample_habitat(pre_sampler));
                let (habitat, dispersal_sampler, turnover_rate, speciation_probability) =
                    scenario.build::<InMemoryAliasDispersalSampler<
                        Self::MathsCore,
                        O::Habitat,
                        Self::Rng,
                    >>();
                let coalescence_sampler = UnconditionalCoalescenceSampler::default();
                let emigration_exit = NeverEmigrationExit::default();
                let event_sampler = UnconditionalEventSampler::default();
                let immigration_entry = NeverImmigrationEntry::default();
                let active_lineage_sampler = IndividualAliasActiveLineageSampler::new(
                    &habitat,
                    &turnover_rate,
                    &lineage_store,
                );

                let mut simulation = SimulationBuilder {
                    maths: PhantomData::<Self::MathsCore>,
                    habitat,
                    lineage_reference: PhantomData::<Self::LineageReference>,
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

                let (status, time, steps) = parallelisation::monolithic::monolithic::simulate(
                    &mut simulation,
                    pause_before,
                    local_partition,
                );

                match status {
                    Status::Done => Ok(AlgorithmResult::Done { time, steps }),
                    Status::Paused => Ok(AlgorithmResult::Paused {
                        time,
                        steps,
                        lineages: simulation
                            .active_lineage_sampler()
                            .iter_active_lineages_ordered(
                                simulation.habitat(),
                                simulation.lineage_store(),
                            )
                            .cloned()
                            .collect(),
                        rng: simulation.rng_mut().clone(),
                        marker: PhantomData,
                    }),
                }
            },
            non_monolithic_parallelism_mode => {
                let decomposition =
                    O::decompose(scenario.habitat(), local_partition.get_partition());

                let rng = rng.split_to_stream(u64::from(local_partition.get_partition().rank()));
                let lineage_store =
                    Self::LineageStore::from_origin_sampler(DecompositionOriginSampler::new(
                        scenario.sample_habitat(pre_sampler),
                        &decomposition,
                    ));
                let (habitat, dispersal_sampler, turnover_rate, speciation_probability) =
                    scenario.build::<InMemoryAliasDispersalSampler<
                        Self::MathsCore,
                        O::Habitat,
                        Self::Rng,
                    >>();
                let coalescence_sampler = UnconditionalCoalescenceSampler::default();
                let emigration_exit = DomainEmigrationExit::new(decomposition);
                let event_sampler = UnconditionalEventSampler::default();
                let immigration_entry = BufferedImmigrationEntry::default();
                let active_lineage_sampler = IndividualAliasActiveLineageSampler::new(
                    &habitat,
                    &turnover_rate,
                    &lineage_store,
                );

                let mut simulation = SimulationBuilder {
                    maths: PhantomData::<Self::MathsCore>,
                    habitat,
                    lineage_reference: PhantomData::<Self::LineageReference>,
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

                let (_status, time, steps) = match non_monolithic_parallelism_mode {
                    ParallelismMode::Monolithic => unsafe { unreachable_unchecked() },
                    ParallelismMode::Optimistic(OptimisticParallelismMode { delta_sync }) => {
                        parallelisation::monolithic::optimistic::simulate(
                            &mut simulation,
                            delta_sync,
                            local_partition,
                        )
                    },
                    ParallelismMode::Lockstep => parallelisation::monolithic::lockstep::simulate(
                        &mut simulation,
                        local_partition,
                    ),
                    ParallelismMode::OptimisticLockstep => {
                        parallelisation::monolithic::optimistic_lockstep::simulate(
                            &mut simulation,
                            local_partition,
                        )
                    },
                    ParallelismMode::Averaging(AveragingParallelismMode { delta_sync }) => {
                        parallelisation::monolithic::averaging::simulate(
                            &mut simulation,
                            delta_sync,
                            local_partition,
                        )
                    },
                };

                // TODO: Adapt for parallel pausing
                Ok(AlgorithmResult::Done { time, steps })
            },
        }
    }
}

// Optimised 'Classical' implementation for the `UniformTurnoverSampler`
#[allow(clippy::type_complexity)]
impl<
        O: Scenario<
            IntrinsicsMathsCore,
            Pcg<IntrinsicsMathsCore>,
            LineageReference = InMemoryLineageReference,
            TurnoverRate = UniformTurnoverRate,
        >,
        R: Reporter,
        P: LocalPartition<R>,
    > Algorithm<O, R, P> for ClassicalAlgorithm
where
    O::LineageStore<ClassicalLineageStore<IntrinsicsMathsCore, O::Habitat>>:
        LocallyCoherentLineageStore<IntrinsicsMathsCore, O::Habitat, InMemoryLineageReference>,
{
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
            ParallelismMode::Monolithic => {
                let lineage_store =
                    Self::LineageStore::from_origin_sampler(scenario.sample_habitat(pre_sampler));
                let (habitat, dispersal_sampler, turnover_rate, speciation_probability) =
                    scenario.build::<InMemoryAliasDispersalSampler<
                        Self::MathsCore,
                        O::Habitat,
                        Pcg<Self::MathsCore>,
                    >>();
                let coalescence_sampler = UnconditionalCoalescenceSampler::default();
                let emigration_exit = NeverEmigrationExit::default();
                let event_sampler = UnconditionalEventSampler::default();
                let immigration_entry = NeverImmigrationEntry::default();
                let active_lineage_sampler = ClassicalActiveLineageSampler::new(&lineage_store);

                let mut simulation = SimulationBuilder {
                    maths: PhantomData::<Self::MathsCore>,
                    habitat,
                    lineage_reference: PhantomData::<Self::LineageReference>,
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

                let (status, time, steps) = parallelisation::monolithic::monolithic::simulate(
                    &mut simulation,
                    pause_before,
                    local_partition,
                );

                match status {
                    Status::Done => Ok(AlgorithmResult::Done { time, steps }),
                    Status::Paused => Ok(AlgorithmResult::Paused {
                        time,
                        steps,
                        lineages: simulation
                            .active_lineage_sampler()
                            .iter_active_lineages_ordered(
                                simulation.habitat(),
                                simulation.lineage_store(),
                            )
                            .cloned()
                            .collect(),
                        rng: simulation.rng_mut().clone(),
                        marker: PhantomData,
                    }),
                }
            },
            non_monolithic_parallelism_mode => {
                let decomposition =
                    O::decompose(scenario.habitat(), local_partition.get_partition());

                let rng = rng.split_to_stream(u64::from(local_partition.get_partition().rank()));
                let lineage_store =
                    Self::LineageStore::from_origin_sampler(DecompositionOriginSampler::new(
                        scenario.sample_habitat(pre_sampler),
                        &decomposition,
                    ));
                let (habitat, dispersal_sampler, turnover_rate, speciation_probability) =
                    scenario.build::<InMemoryAliasDispersalSampler<
                        Self::MathsCore,
                        O::Habitat,
                        Pcg<Self::MathsCore>,
                    >>();
                let coalescence_sampler = UnconditionalCoalescenceSampler::default();
                let emigration_exit = DomainEmigrationExit::new(decomposition);
                let event_sampler = UnconditionalEventSampler::default();
                let immigration_entry = BufferedImmigrationEntry::default();
                let active_lineage_sampler = ClassicalActiveLineageSampler::new(&lineage_store);

                let mut simulation = SimulationBuilder {
                    maths: PhantomData::<Self::MathsCore>,
                    habitat,
                    lineage_reference: PhantomData::<Self::LineageReference>,
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

                let (_status, time, steps) = match non_monolithic_parallelism_mode {
                    ParallelismMode::Monolithic => unsafe { unreachable_unchecked() },
                    ParallelismMode::Optimistic(OptimisticParallelismMode { delta_sync }) => {
                        parallelisation::monolithic::optimistic::simulate(
                            &mut simulation,
                            delta_sync,
                            local_partition,
                        )
                    },
                    ParallelismMode::Lockstep => parallelisation::monolithic::lockstep::simulate(
                        &mut simulation,
                        local_partition,
                    ),
                    ParallelismMode::OptimisticLockstep => {
                        parallelisation::monolithic::optimistic_lockstep::simulate(
                            &mut simulation,
                            local_partition,
                        )
                    },
                    ParallelismMode::Averaging(AveragingParallelismMode { delta_sync }) => {
                        parallelisation::monolithic::averaging::simulate(
                            &mut simulation,
                            delta_sync,
                            local_partition,
                        )
                    },
                };

                // TODO: Adapt for parallel pausing
                Ok(AlgorithmResult::Done { time, steps })
            },
        }
    }
}
