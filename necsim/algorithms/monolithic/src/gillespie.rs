use std::{hint::unreachable_unchecked, marker::PhantomData};

use necsim_core::{
    cogs::{GloballyCoherentLineageStore, Habitat, LineageStore, RngCore, SplittableRng},
    simulation::Simulation,
};

use necsim_impls_no_std::{
    cogs::{
        coalescence_sampler::unconditional::UnconditionalCoalescenceSampler,
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
    partitioning::LocalPartition,
    reporter::ReporterContext,
};
use necsim_impls_std::cogs::{
    active_lineage_sampler::gillespie::GillespieActiveLineageSampler, rng::pcg::Pcg,
};

use necsim_algorithms::{Algorithm, AlgorithmArguments};
use necsim_scenarios::Scenario;

use crate::{
    arguments::{
        AveragingParallelismMode, MonolithicArguments, OptimisticParallelismMode, ParallelismMode,
    },
    parallelism,
};

#[allow(clippy::module_name_repetitions, clippy::empty_enum)]
pub struct GillespieAlgorithm {}

impl AlgorithmArguments for GillespieAlgorithm {
    type Arguments = MonolithicArguments;
}

#[allow(clippy::type_complexity)]
impl<
        H: Habitat,
        O: Scenario<
            Pcg,
            GillespieLineageStore<H>,
            Habitat = H,
            LineageReference = InMemoryLineageReference,
        >,
    > Algorithm<GillespieLineageStore<H>, O> for GillespieAlgorithm
where
    O::LineageStore: GloballyCoherentLineageStore<H, InMemoryLineageReference>,
{
    type Error = !;
    type LineageReference = InMemoryLineageReference;
    type LineageStore = O::LineageStore;
    type Rng = Pcg;

    #[allow(clippy::shadow_unrelated, clippy::too_many_lines)]
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
        let mut rng = match args.parallelism_mode {
            ParallelismMode::Monolithic => Pcg::seed_from_u64(seed),
            _ => Pcg::seed_from_u64(seed)
                .split_to_stream(u64::from(local_partition.get_partition_rank())),
        };

        let origin_sampler = scenario.sample_habitat(pre_sampler);

        let decomposition = scenario.decompose(
            local_partition.get_partition_rank(),
            local_partition.get_number_of_partitions(),
        );

        let lineage_store = match args.parallelism_mode {
            ParallelismMode::Monolithic => Self::LineageStore::from_origin_sampler(origin_sampler),
            _ => Self::LineageStore::from_origin_sampler(DecompositionOriginSampler::new(
                origin_sampler,
                &decomposition,
            )),
        };

        let (habitat, dispersal_sampler, turnover_rate, speciation_probability) = scenario.build();

        let coalescence_sampler = UnconditionalCoalescenceSampler::default();

        match args.parallelism_mode {
            ParallelismMode::Monolithic => {
                let emigration_exit = NeverEmigrationExit::default();
                let event_sampler = UnconditionalGillespieEventSampler::default();

                // Pack a PartialSimulation to initialise the GillespieActiveLineageSampler
                let partial_simulation = GillespiePartialSimulation {
                    habitat,
                    speciation_probability,
                    dispersal_sampler,
                    lineage_reference: PhantomData::<Self::LineageReference>,
                    lineage_store,
                    coalescence_sampler,
                    turnover_rate,
                    _rng: PhantomData::<Pcg>,
                };

                let immigration_entry = NeverImmigrationEntry::default();
                let active_lineage_sampler = GillespieActiveLineageSampler::new(
                    &partial_simulation,
                    &event_sampler,
                    &mut rng,
                );

                // Unpack the PartialSimulation to create the full Simulation
                let GillespiePartialSimulation {
                    habitat,
                    speciation_probability,
                    dispersal_sampler,
                    lineage_reference: _,
                    lineage_store,
                    coalescence_sampler,
                    turnover_rate,
                    _rng: _,
                } = partial_simulation;

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

                Ok(parallelism::monolithic::simulate(
                    simulation,
                    local_partition,
                ))
            },
            non_monolithic_parallelism_mode => {
                let emigration_exit = DomainEmigrationExit::new(decomposition);
                let event_sampler = UnconditionalGillespieEventSampler::default();

                // Pack a PartialSimulation to initialise the GillespieActiveLineageSampler
                let partial_simulation = GillespiePartialSimulation {
                    habitat,
                    speciation_probability,
                    dispersal_sampler,
                    lineage_reference: PhantomData::<Self::LineageReference>,
                    lineage_store,
                    coalescence_sampler,
                    turnover_rate,
                    _rng: PhantomData::<Pcg>,
                };

                let immigration_entry = BufferedImmigrationEntry::default();
                let active_lineage_sampler = GillespieActiveLineageSampler::new(
                    &partial_simulation,
                    &event_sampler,
                    &mut rng,
                );

                // Unpack the PartialSimulation to create the full Simulation
                let GillespiePartialSimulation {
                    habitat,
                    speciation_probability,
                    dispersal_sampler,
                    lineage_reference: _,
                    lineage_store,
                    coalescence_sampler,
                    turnover_rate,
                    _rng: _,
                } = partial_simulation;

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
                    ParallelismMode::Optimistic(OptimisticParallelismMode { delta_sync }) => Ok(
                        parallelism::optimistic::simulate(simulation, local_partition, delta_sync),
                    ),
                    ParallelismMode::Lockstep => {
                        Ok(parallelism::lockstep::simulate(simulation, local_partition))
                    },
                    ParallelismMode::OptimisticLockstep => Ok(
                        parallelism::optimistic_lockstep::simulate(simulation, local_partition),
                    ),
                    ParallelismMode::Averaging(AveragingParallelismMode { delta_sync }) => Ok(
                        parallelism::averaging::simulate(simulation, local_partition, delta_sync),
                    ),
                }
            },
        }
    }
}
