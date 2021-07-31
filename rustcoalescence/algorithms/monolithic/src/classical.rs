use std::{hint::unreachable_unchecked, marker::PhantomData};

use necsim_core::{
    cogs::{LineageStore, LocallyCoherentLineageStore, RngCore, SplittableRng},
    reporter::Reporter,
    simulation::Simulation,
};
use necsim_core_bond::NonNegativeF64;

use necsim_impls_no_std::{
    cogs::{
        active_lineage_sampler::classical::ClassicalActiveLineageSampler,
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
    parallelisation,
};
use necsim_impls_std::cogs::rng::pcg::Pcg;
use necsim_partitioning_core::LocalPartition;

use rustcoalescence_algorithms::{Algorithm, AlgorithmArguments};
use rustcoalescence_scenarios::Scenario;

use crate::arguments::{
    AveragingParallelismMode, MonolithicArguments, OptimisticParallelismMode, ParallelismMode,
};

#[allow(clippy::module_name_repetitions, clippy::empty_enum)]
pub enum ClassicalAlgorithm {}

impl AlgorithmArguments for ClassicalAlgorithm {
    type Arguments = MonolithicArguments;
}

#[allow(clippy::type_complexity)]
impl<
        O: Scenario<
            Pcg,
            LineageReference = InMemoryLineageReference,
            TurnoverRate = UniformTurnoverRate,
        >,
        R: Reporter,
    > Algorithm<O, R> for ClassicalAlgorithm
where
    O::LineageStore<ClassicalLineageStore<O::Habitat>>:
        LocallyCoherentLineageStore<O::Habitat, InMemoryLineageReference>,
{
    type Error = !;
    type LineageReference = InMemoryLineageReference;
    type LineageStore = O::LineageStore<ClassicalLineageStore<O::Habitat>>;
    type Rng = Pcg;

    fn initialise_and_simulate<I: Iterator<Item = u64>, P: LocalPartition<R>>(
        args: Self::Arguments,
        seed: u64,
        scenario: O,
        pre_sampler: OriginPreSampler<I>,
        local_partition: &mut P,
    ) -> Result<(NonNegativeF64, u64), Self::Error> {
        match args.parallelism_mode {
            ParallelismMode::Monolithic => {
                let rng = Pcg::seed_from_u64(seed);
                let lineage_store =
                    Self::LineageStore::from_origin_sampler(scenario.sample_habitat(pre_sampler));
                let (habitat, dispersal_sampler, turnover_rate, speciation_probability) =
                    scenario.build::<InMemoryAliasDispersalSampler<O::Habitat, Pcg>>();
                let coalescence_sampler = UnconditionalCoalescenceSampler::default();
                let emigration_exit = NeverEmigrationExit::default();
                let event_sampler = UnconditionalEventSampler::default();
                let immigration_entry = NeverImmigrationEntry::default();
                let active_lineage_sampler = ClassicalActiveLineageSampler::new(&lineage_store);

                let simulation = Simulation::builder()
                    .habitat(habitat)
                    .rng(rng)
                    .speciation_probability(speciation_probability)
                    .dispersal_sampler(dispersal_sampler)
                    .lineage_reference(PhantomData::<Self::LineageReference>)
                    .lineage_store(lineage_store)
                    .emigration_exit(emigration_exit)
                    .coalescence_sampler(coalescence_sampler)
                    .turnover_rate(turnover_rate)
                    .event_sampler(event_sampler)
                    .immigration_entry(immigration_entry)
                    .active_lineage_sampler(active_lineage_sampler)
                    .build();

                Ok(parallelisation::monolithic::monolithic::simulate(
                    simulation,
                    local_partition,
                ))
            },
            non_monolithic_parallelism_mode => {
                let decomposition = O::decompose(
                    scenario.habitat(),
                    local_partition.get_partition_rank(),
                    local_partition.get_number_of_partitions(),
                );

                let rng = Pcg::seed_from_u64(seed)
                    .split_to_stream(u64::from(local_partition.get_partition_rank()));
                let lineage_store =
                    Self::LineageStore::from_origin_sampler(DecompositionOriginSampler::new(
                        scenario.sample_habitat(pre_sampler),
                        &decomposition,
                    ));
                let (habitat, dispersal_sampler, turnover_rate, speciation_probability) =
                    scenario.build::<InMemoryAliasDispersalSampler<O::Habitat, Pcg>>();
                let coalescence_sampler = UnconditionalCoalescenceSampler::default();
                let emigration_exit = DomainEmigrationExit::new(decomposition);
                let event_sampler = UnconditionalEventSampler::default();
                let immigration_entry = BufferedImmigrationEntry::default();
                let active_lineage_sampler = ClassicalActiveLineageSampler::new(&lineage_store);

                let simulation = Simulation::builder()
                    .habitat(habitat)
                    .rng(rng)
                    .speciation_probability(speciation_probability)
                    .dispersal_sampler(dispersal_sampler)
                    .lineage_reference(PhantomData::<Self::LineageReference>)
                    .lineage_store(lineage_store)
                    .emigration_exit(emigration_exit)
                    .coalescence_sampler(coalescence_sampler)
                    .turnover_rate(turnover_rate)
                    .event_sampler(event_sampler)
                    .immigration_entry(immigration_entry)
                    .active_lineage_sampler(active_lineage_sampler)
                    .build();

                match non_monolithic_parallelism_mode {
                    ParallelismMode::Monolithic => unsafe { unreachable_unchecked() },
                    ParallelismMode::Optimistic(OptimisticParallelismMode { delta_sync }) => {
                        Ok(parallelisation::monolithic::optimistic::simulate(
                            simulation,
                            delta_sync,
                            local_partition,
                        ))
                    },
                    ParallelismMode::Lockstep => {
                        Ok(parallelisation::monolithic::lockstep::simulate(
                            simulation,
                            local_partition,
                        ))
                    },
                    ParallelismMode::OptimisticLockstep => {
                        Ok(parallelisation::monolithic::optimistic_lockstep::simulate(
                            simulation,
                            local_partition,
                        ))
                    },
                    ParallelismMode::Averaging(AveragingParallelismMode { delta_sync }) => {
                        Ok(parallelisation::monolithic::averaging::simulate(
                            simulation,
                            delta_sync,
                            local_partition,
                        ))
                    },
                }
            },
        }
    }
}
