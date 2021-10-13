use std::{hint::unreachable_unchecked, marker::PhantomData};

use necsim_core::{
    cogs::{GloballyCoherentLineageStore, LineageStore, SeedableRng, SplittableRng},
    reporter::Reporter,
    simulation::SimulationBuilder,
};
use necsim_core_bond::NonNegativeF64;

use necsim_core_f64::IntrinsicsF64Core;
use necsim_impls_no_std::{
    cogs::{
        coalescence_sampler::unconditional::UnconditionalCoalescenceSampler,
        dispersal_sampler::in_memory::alias::InMemoryAliasDispersalSampler,
        emigration_exit::{domain::DomainEmigrationExit, never::NeverEmigrationExit},
        event_sampler::gillespie::{
            unconditional::UnconditionalGillespieEventSampler, GillespiePartialSimulation,
        },
        immigration_entry::{buffered::BufferedImmigrationEntry, never::NeverImmigrationEntry},
        lineage_reference::in_memory::InMemoryLineageReference,
        lineage_store::coherent::globally::gillespie::GillespieLineageStore,
        origin_sampler::{
            decomposition::DecompositionOriginSampler, pre_sampler::OriginPreSampler,
        },
    },
    parallelisation,
};
use necsim_impls_std::cogs::{
    active_lineage_sampler::gillespie::GillespieActiveLineageSampler, rng::pcg::Pcg,
};
use necsim_partitioning_core::LocalPartition;

use rustcoalescence_algorithms::{Algorithm, AlgorithmArguments};
use rustcoalescence_scenarios::Scenario;

use crate::arguments::{
    AveragingParallelismMode, MonolithicArguments, OptimisticParallelismMode, ParallelismMode,
};

#[allow(clippy::module_name_repetitions, clippy::empty_enum)]
pub struct GillespieAlgorithm {}

impl AlgorithmArguments for GillespieAlgorithm {
    type Arguments = MonolithicArguments;
}

#[allow(clippy::type_complexity)]
impl<
        O: Scenario<
            IntrinsicsF64Core,
            Pcg<IntrinsicsF64Core>,
            LineageReference = InMemoryLineageReference,
        >,
        R: Reporter,
        P: LocalPartition<R>,
    > Algorithm<O, R, P> for GillespieAlgorithm
where
    O::LineageStore<GillespieLineageStore<IntrinsicsF64Core, O::Habitat>>:
        GloballyCoherentLineageStore<IntrinsicsF64Core, O::Habitat, InMemoryLineageReference>,
{
    type Error = !;
    type F64Core = IntrinsicsF64Core;
    type LineageReference = InMemoryLineageReference;
    type LineageStore = O::LineageStore<GillespieLineageStore<Self::F64Core, O::Habitat>>;
    type Rng = Pcg<Self::F64Core>;

    #[allow(clippy::shadow_unrelated, clippy::too_many_lines)]
    fn initialise_and_simulate<I: Iterator<Item = u64>>(
        args: Self::Arguments,
        seed: u64,
        scenario: O,
        pre_sampler: OriginPreSampler<Self::F64Core, I>,
        local_partition: &mut P,
    ) -> Result<(NonNegativeF64, u64), Self::Error> {
        match args.parallelism_mode {
            ParallelismMode::Monolithic => {
                let mut rng = Pcg::seed_from_u64(seed);
                let lineage_store =
                    Self::LineageStore::from_origin_sampler(scenario.sample_habitat(pre_sampler));
                let (habitat, dispersal_sampler, turnover_rate, speciation_probability) =
                    scenario.build::<InMemoryAliasDispersalSampler<
                        Self::F64Core,
                        O::Habitat,
                        Pcg<Self::F64Core>,
                    >>();
                let coalescence_sampler = UnconditionalCoalescenceSampler::default();
                let emigration_exit = NeverEmigrationExit::default();
                let event_sampler = UnconditionalGillespieEventSampler::default();
                let immigration_entry = NeverImmigrationEntry::default();

                // Pack a PartialSimulation to initialise the GillespieActiveLineageSampler
                let partial_simulation = GillespiePartialSimulation {
                    f64_core: PhantomData::<Self::F64Core>,
                    habitat,
                    speciation_probability,
                    dispersal_sampler,
                    lineage_reference: PhantomData::<Self::LineageReference>,
                    lineage_store,
                    coalescence_sampler,
                    turnover_rate,
                    _rng: PhantomData::<Pcg<Self::F64Core>>,
                };

                let active_lineage_sampler = GillespieActiveLineageSampler::new(
                    &partial_simulation,
                    &event_sampler,
                    &mut rng,
                );

                // Unpack the PartialSimulation to create the full Simulation
                let GillespiePartialSimulation {
                    f64_core: _,
                    habitat,
                    speciation_probability,
                    dispersal_sampler,
                    lineage_reference: _,
                    lineage_store,
                    coalescence_sampler,
                    turnover_rate,
                    _rng: _,
                } = partial_simulation;

                let simulation = SimulationBuilder {
                    f64_core: PhantomData::<Self::F64Core>,
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

                let mut rng = Pcg::seed_from_u64(seed)
                    .split_to_stream(u64::from(local_partition.get_partition_rank()));
                let lineage_store =
                    Self::LineageStore::from_origin_sampler(DecompositionOriginSampler::new(
                        scenario.sample_habitat(pre_sampler),
                        &decomposition,
                    ));
                let (habitat, dispersal_sampler, turnover_rate, speciation_probability) =
                    scenario.build::<InMemoryAliasDispersalSampler<
                        Self::F64Core,
                        O::Habitat,
                        Pcg<Self::F64Core>,
                    >>();
                let coalescence_sampler = UnconditionalCoalescenceSampler::default();
                let emigration_exit = DomainEmigrationExit::new(decomposition);
                let event_sampler = UnconditionalGillespieEventSampler::default();
                let immigration_entry = BufferedImmigrationEntry::default();

                // Pack a PartialSimulation to initialise the GillespieActiveLineageSampler
                let partial_simulation = GillespiePartialSimulation {
                    f64_core: PhantomData::<Self::F64Core>,
                    habitat,
                    speciation_probability,
                    dispersal_sampler,
                    lineage_reference: PhantomData::<Self::LineageReference>,
                    lineage_store,
                    coalescence_sampler,
                    turnover_rate,
                    _rng: PhantomData::<Pcg<Self::F64Core>>,
                };

                let active_lineage_sampler = GillespieActiveLineageSampler::new(
                    &partial_simulation,
                    &event_sampler,
                    &mut rng,
                );

                // Unpack the PartialSimulation to create the full Simulation
                let GillespiePartialSimulation {
                    f64_core: _,
                    habitat,
                    speciation_probability,
                    dispersal_sampler,
                    lineage_reference: _,
                    lineage_store,
                    coalescence_sampler,
                    turnover_rate,
                    _rng: _,
                } = partial_simulation;

                let simulation = SimulationBuilder {
                    f64_core: PhantomData::<Self::F64Core>,
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
