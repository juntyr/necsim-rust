use std::{hint::unreachable_unchecked, marker::PhantomData};

use necsim_core::{
    cogs::{
        ActiveLineageSampler, GloballyCoherentLineageStore, SeparableDispersalSampler,
        SplittableRng,
    },
    reporter::Reporter,
    simulation::SimulationBuilder,
};
use necsim_core_bond::NonNegativeF64;
use necsim_core_maths::IntrinsicsMathsCore;

use necsim_impls_no_std::{
    cogs::{
        active_lineage_sampler::alias::location::LocationAliasActiveLineageSampler,
        coalescence_sampler::conditional::ConditionalCoalescenceSampler,
        dispersal_sampler::in_memory::separable_alias::InMemorySeparableAliasDispersalSampler,
        emigration_exit::{domain::DomainEmigrationExit, never::NeverEmigrationExit},
        event_sampler::gillespie::conditional::ConditionalGillespieEventSampler,
        immigration_entry::{buffered::BufferedImmigrationEntry, never::NeverImmigrationEntry},
        lineage_reference::in_memory::InMemoryLineageReference,
        lineage_store::coherent::globally::gillespie::GillespieLineageStore,
        origin_sampler::{
            decomposition::DecompositionOriginSampler, pre_sampler::OriginPreSampler,
        },
    },
    parallelisation::{self, Status},
};
use necsim_impls_std::cogs::rng::pcg::Pcg;
use necsim_partitioning_core::LocalPartition;

use rustcoalescence_algorithms::{Algorithm, AlgorithmParamters, AlgorithmResult};
use rustcoalescence_scenarios::Scenario;

use crate::arguments::{
    AveragingParallelismMode, MonolithicArguments, OptimisticParallelismMode, ParallelismMode,
};

#[allow(clippy::module_name_repetitions, clippy::empty_enum)]
pub struct EventSkippingAlgorithm {}

impl AlgorithmParamters for EventSkippingAlgorithm {
    type Arguments = MonolithicArguments;
    type Error = !;
}

#[allow(clippy::type_complexity)]
impl<
        O: Scenario<
            IntrinsicsMathsCore,
            Pcg<IntrinsicsMathsCore>,
            LineageReference = InMemoryLineageReference,
        >,
        R: Reporter,
        P: LocalPartition<R>,
    > Algorithm<O, R, P> for EventSkippingAlgorithm
where
    O::LineageStore<GillespieLineageStore<IntrinsicsMathsCore, O::Habitat>>:
        GloballyCoherentLineageStore<IntrinsicsMathsCore, O::Habitat, InMemoryLineageReference>,
    O::DispersalSampler<
        InMemorySeparableAliasDispersalSampler<
            IntrinsicsMathsCore,
            O::Habitat,
            Pcg<IntrinsicsMathsCore>,
        >,
    >: SeparableDispersalSampler<IntrinsicsMathsCore, O::Habitat, Pcg<IntrinsicsMathsCore>>,
{
    type LineageReference = InMemoryLineageReference;
    type LineageStore = O::LineageStore<GillespieLineageStore<Self::MathsCore, O::Habitat>>;
    type MathsCore = IntrinsicsMathsCore;
    type Rng = Pcg<Self::MathsCore>;

    #[allow(clippy::shadow_unrelated, clippy::too_many_lines)]
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
                let (
                    habitat,
                    dispersal_sampler,
                    turnover_rate,
                    speciation_probability,
                    origin_sampler_auxiliary,
                    _decomposition_auxiliary,
                ) = scenario.build::<InMemorySeparableAliasDispersalSampler<
                    Self::MathsCore,
                    O::Habitat,
                    Self::Rng,
                >>();
                let coalescence_sampler = ConditionalCoalescenceSampler::default();
                let event_sampler = ConditionalGillespieEventSampler::default();

                let (lineage_store, active_lineage_sampler): (Self::LineageStore, _) =
                    LocationAliasActiveLineageSampler::init_with_store(
                        O::sample_habitat(&habitat, pre_sampler, origin_sampler_auxiliary),
                        &dispersal_sampler,
                        &coalescence_sampler,
                        &turnover_rate,
                        &speciation_probability,
                        &event_sampler,
                    );

                let emigration_exit = NeverEmigrationExit::default();
                let immigration_entry = NeverImmigrationEntry::default();

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
                let rng = rng.split_to_stream(u64::from(local_partition.get_partition().rank()));

                let (
                    habitat,
                    dispersal_sampler,
                    turnover_rate,
                    speciation_probability,
                    origin_sampler_auxiliary,
                    decomposition_auxiliary,
                ) = scenario.build::<InMemorySeparableAliasDispersalSampler<
                    Self::MathsCore,
                    O::Habitat,
                    Self::Rng,
                >>();
                let coalescence_sampler = ConditionalCoalescenceSampler::default();
                let event_sampler = ConditionalGillespieEventSampler::default();

                let decomposition = O::decompose(
                    &habitat,
                    local_partition.get_partition(),
                    decomposition_auxiliary,
                );
                let origin_sampler = DecompositionOriginSampler::new(
                    O::sample_habitat(&habitat, pre_sampler, origin_sampler_auxiliary),
                    &decomposition,
                );

                let (lineage_store, active_lineage_sampler): (Self::LineageStore, _) =
                    LocationAliasActiveLineageSampler::init_with_store(
                        origin_sampler,
                        &dispersal_sampler,
                        &coalescence_sampler,
                        &turnover_rate,
                        &speciation_probability,
                        &event_sampler,
                    );

                let emigration_exit = DomainEmigrationExit::new(decomposition);
                let immigration_entry = BufferedImmigrationEntry::default();

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
