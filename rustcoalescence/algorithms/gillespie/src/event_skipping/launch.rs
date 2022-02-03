use std::{hint::unreachable_unchecked, marker::PhantomData};

use necsim_core::{
    cogs::{
        ActiveLineageSampler, GloballyCoherentLineageStore, MathsCore, SeparableDispersalSampler,
        SplittableRng,
    },
    reporter::Reporter,
    simulation::SimulationBuilder,
};
use necsim_core_bond::NonNegativeF64;

use necsim_impls_no_std::{
    cogs::{
        coalescence_sampler::conditional::ConditionalCoalescenceSampler,
        dispersal_sampler::in_memory::separable_alias::InMemorySeparableAliasDispersalSampler,
        emigration_exit::{domain::DomainEmigrationExit, never::NeverEmigrationExit},
        immigration_entry::{buffered::BufferedImmigrationEntry, never::NeverImmigrationEntry},
        lineage_reference::in_memory::InMemoryLineageReference,
        lineage_store::coherent::globally::gillespie::GillespieLineageStore,
        origin_sampler::{
            decomposition::DecompositionOriginSampler, pre_sampler::OriginPreSampler,
        },
    },
    parallelisation::{self, Status},
};
use necsim_partitioning_core::LocalPartition;

use rustcoalescence_algorithms::result::SimulationOutcome;
use rustcoalescence_scenarios::Scenario;

use super::initialiser::EventSkippingLineageStoreSampleInitialiser;
use crate::arguments::{
    AveragingParallelismMode, MonolithicArguments, OptimisticParallelismMode, ParallelismMode,
};

#[allow(clippy::shadow_unrelated, clippy::too_many_lines)]
pub fn initialise_and_simulate<
    M: MathsCore,
    G: SplittableRng<M>,
    O: Scenario<M, G, LineageReference = InMemoryLineageReference>,
    R: Reporter,
    P: LocalPartition<R>,
    I: Iterator<Item = u64>,
    L: EventSkippingLineageStoreSampleInitialiser<M, G, O, Error>,
    Error,
>(
    args: MonolithicArguments,
    rng: G,
    scenario: O,
    pre_sampler: OriginPreSampler<M, I>,
    pause_before: Option<NonNegativeF64>,
    local_partition: &mut P,
    lineage_store_sampler_initialiser: L,
) -> Result<SimulationOutcome<M, G>, Error>
where
    O::LineageStore<GillespieLineageStore<M, O::Habitat>>:
        GloballyCoherentLineageStore<M, O::Habitat, InMemoryLineageReference>,
    O::DispersalSampler<InMemorySeparableAliasDispersalSampler<M, O::Habitat, G>>:
        SeparableDispersalSampler<M, O::Habitat, G>,
{
    match args.parallelism_mode {
        ParallelismMode::Monolithic => {
            let (
                habitat,
                dispersal_sampler,
                turnover_rate,
                speciation_probability,
                origin_sampler_auxiliary,
                _decomposition_auxiliary,
            ) = scenario.build::<InMemorySeparableAliasDispersalSampler<M, O::Habitat, G>>();
            let coalescence_sampler = ConditionalCoalescenceSampler::default();

            let (lineage_store, dispersal_sampler, event_sampler, active_lineage_sampler): (
                O::LineageStore<GillespieLineageStore<M, O::Habitat>>,
                _,
                _,
                _,
            ) = lineage_store_sampler_initialiser.init(
                O::sample_habitat(&habitat, pre_sampler, origin_sampler_auxiliary),
                dispersal_sampler,
                &coalescence_sampler,
                &turnover_rate,
                &speciation_probability,
                local_partition,
            )?;

            let emigration_exit = NeverEmigrationExit::default();
            let immigration_entry = NeverImmigrationEntry::default();

            let mut simulation = SimulationBuilder {
                maths: PhantomData::<M>,
                habitat,
                lineage_reference: PhantomData::<InMemoryLineageReference>,
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
                Status::Done => Ok(SimulationOutcome::Done { time, steps }),
                Status::Paused => Ok(SimulationOutcome::Paused {
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
            ) = scenario.build::<InMemorySeparableAliasDispersalSampler<M, O::Habitat, G>>();
            let coalescence_sampler = ConditionalCoalescenceSampler::default();

            let decomposition = O::decompose(
                &habitat,
                local_partition.get_partition(),
                decomposition_auxiliary,
            );
            let origin_sampler = DecompositionOriginSampler::new(
                O::sample_habitat(&habitat, pre_sampler, origin_sampler_auxiliary),
                &decomposition,
            );

            let (lineage_store, dispersal_sampler, event_sampler, active_lineage_sampler): (
                O::LineageStore<GillespieLineageStore<M, O::Habitat>>,
                _,
                _,
                _,
            ) = lineage_store_sampler_initialiser.init(
                origin_sampler,
                dispersal_sampler,
                &coalescence_sampler,
                &turnover_rate,
                &speciation_probability,
                local_partition,
            )?;

            let emigration_exit = DomainEmigrationExit::new(decomposition);
            let immigration_entry = BufferedImmigrationEntry::default();

            let mut simulation = SimulationBuilder {
                maths: PhantomData::<M>,
                habitat,
                lineage_reference: PhantomData::<InMemoryLineageReference>,
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
            Ok(SimulationOutcome::Done { time, steps })
        },
    }
}
