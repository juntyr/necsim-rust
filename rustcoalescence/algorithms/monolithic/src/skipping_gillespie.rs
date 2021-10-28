use std::{hint::unreachable_unchecked, marker::PhantomData};

use necsim_core::{
    cogs::{GloballyCoherentLineageStore, LineageStore, SeparableDispersalSampler, SplittableRng},
    reporter::Reporter,
    simulation::SimulationBuilder,
};
use necsim_core_bond::NonNegativeF64;
use necsim_core_maths::IntrinsicsMathsCore;

use necsim_impls_no_std::{
    cogs::{
        coalescence_sampler::conditional::ConditionalCoalescenceSampler,
        dispersal_sampler::in_memory::separable_alias::InMemorySeparableAliasDispersalSampler,
        emigration_exit::{domain::DomainEmigrationExit, never::NeverEmigrationExit},
        event_sampler::gillespie::{
            conditional::ConditionalGillespieEventSampler, GillespiePartialSimulation,
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
pub struct SkippingGillespieAlgorithm {}

impl AlgorithmArguments for SkippingGillespieAlgorithm {
    type Arguments = MonolithicArguments;
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
    > Algorithm<O, R, P> for SkippingGillespieAlgorithm
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
    type Error = !;
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
        local_partition: &mut P,
    ) -> Result<(NonNegativeF64, u64), Self::Error> {
        match args.parallelism_mode {
            ParallelismMode::Monolithic => {
                let mut rng = rng;
                let lineage_store =
                    Self::LineageStore::from_origin_sampler(scenario.sample_habitat(pre_sampler));
                let (habitat, dispersal_sampler, turnover_rate, speciation_probability) =
                    scenario.build::<InMemorySeparableAliasDispersalSampler<
                        Self::MathsCore,
                        O::Habitat,
                        Pcg<Self::MathsCore>,
                    >>();
                let coalescence_sampler = ConditionalCoalescenceSampler::default();
                let emigration_exit = NeverEmigrationExit::default();
                let event_sampler = ConditionalGillespieEventSampler::default();
                let immigration_entry = NeverImmigrationEntry::default();

                // Pack a PartialSimulation to initialise the GillespieActiveLineageSampler
                let partial_simulation = GillespiePartialSimulation {
                    maths: PhantomData::<Self::MathsCore>,
                    habitat,
                    speciation_probability,
                    dispersal_sampler,
                    lineage_reference: PhantomData::<Self::LineageReference>,
                    lineage_store,
                    coalescence_sampler,
                    turnover_rate,
                    _rng: PhantomData::<Pcg<Self::MathsCore>>,
                };

                let active_lineage_sampler = GillespieActiveLineageSampler::new(
                    &partial_simulation,
                    &event_sampler,
                    &mut rng,
                );

                // Unpack the PartialSimulation to create the full Simulation
                let GillespiePartialSimulation {
                    maths: _,
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

                Ok(parallelisation::monolithic::monolithic::simulate(
                    simulation,
                    local_partition,
                ))
            },
            non_monolithic_parallelism_mode => {
                let decomposition =
                    O::decompose(scenario.habitat(), local_partition.get_partition());

                let mut rng =
                    rng.split_to_stream(u64::from(local_partition.get_partition().rank()));
                let lineage_store =
                    Self::LineageStore::from_origin_sampler(DecompositionOriginSampler::new(
                        scenario.sample_habitat(pre_sampler),
                        &decomposition,
                    ));
                let (habitat, dispersal_sampler, turnover_rate, speciation_probability) =
                    scenario.build::<InMemorySeparableAliasDispersalSampler<
                        Self::MathsCore,
                        O::Habitat,
                        Pcg<Self::MathsCore>,
                    >>();
                let coalescence_sampler = ConditionalCoalescenceSampler::default();
                let emigration_exit = DomainEmigrationExit::new(decomposition);
                let event_sampler = ConditionalGillespieEventSampler::default();
                let immigration_entry = BufferedImmigrationEntry::default();

                // Pack a PartialSimulation to initialise the GillespieActiveLineageSampler
                let partial_simulation = GillespiePartialSimulation {
                    maths: PhantomData::<Self::MathsCore>,
                    habitat,
                    speciation_probability,
                    dispersal_sampler,
                    lineage_reference: PhantomData::<Self::LineageReference>,
                    lineage_store,
                    coalescence_sampler,
                    turnover_rate,
                    _rng: PhantomData::<Pcg<Self::MathsCore>>,
                };

                let active_lineage_sampler = GillespieActiveLineageSampler::new(
                    &partial_simulation,
                    &event_sampler,
                    &mut rng,
                );

                // Unpack the PartialSimulation to create the full Simulation
                let GillespiePartialSimulation {
                    maths: _,
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
