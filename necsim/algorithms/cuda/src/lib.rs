#![deny(clippy::pedantic)]

#[macro_use]
extern crate serde_derive_state;

use std::{collections::VecDeque, convert::TryInto};

use arguments::{CudaArguments, IsolatedParallelismMode, ParallelismMode};
use necsim_core::{
    cogs::RngCore,
    lineage::{GlobalLineageReference, Lineage},
    reporter::Reporter,
    simulation::Simulation,
};

use necsim_impls_cuda::{cogs::rng::CudaRng, event_buffer::EventBuffer, value_buffer::ValueBuffer};
use necsim_impls_no_std::{
    cogs::{
        active_lineage_sampler::independent::{
            event_time_sampler::exp::ExpEventTimeSampler, IndependentActiveLineageSampler,
        },
        coalescence_sampler::independent::IndependentCoalescenceSampler,
        dispersal_sampler::in_memory::packed_alias::InMemoryPackedAliasDispersalSampler,
        emigration_exit::never::NeverEmigrationExit,
        event_sampler::independent::IndependentEventSampler,
        immigration_entry::never::NeverImmigrationEntry,
        lineage_store::independent::IndependentLineageStore,
        origin_sampler::{
            decomposition::DecompositionOriginSampler, pre_sampler::OriginPreSampler,
        },
        rng::fixedseahash::FixedSeaHash,
    },
    partitioning::LocalPartition,
    reporter::ReporterContext,
};

use necsim_algorithms::{Algorithm, AlgorithmArguments};
use necsim_scenarios::Scenario;

use rust_cuda::{common::RustToCuda, host::CudaDropWrapper};
use rustacuda::{
    function::{BlockSize, FunctionAttribute, GridSize},
    prelude::{Stream, StreamFlags},
};

mod arguments;
mod cuda;
mod info;
mod kernel;
mod simulate;

use crate::kernel::SimulationKernel;
use cuda::with_initialised_cuda;
use simulate::simulate;

#[allow(clippy::module_name_repetitions, clippy::empty_enum)]
pub enum CudaAlgorithm {}

impl AlgorithmArguments for CudaAlgorithm {
    type Arguments = CudaArguments;
}

#[allow(clippy::type_complexity)]
impl<O: Scenario<CudaRng<FixedSeaHash>>> Algorithm<O> for CudaAlgorithm
where
    O::Habitat: RustToCuda,
    O::DispersalSampler<InMemoryPackedAliasDispersalSampler<O::Habitat, CudaRng<FixedSeaHash>>>:
        RustToCuda,
    O::TurnoverRate: RustToCuda,
    O::SpeciationProbability: RustToCuda,
{
    type Error = anyhow::Error;
    type LineageReference = GlobalLineageReference;
    type LineageStore = IndependentLineageStore<O::Habitat>;
    type Rng = CudaRng<FixedSeaHash>;

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
        };

        let (habitat, dispersal_sampler, turnover_rate, speciation_probability) =
            scenario
                .build::<InMemoryPackedAliasDispersalSampler<O::Habitat, CudaRng<FixedSeaHash>>>();
        let rng = CudaRng::from(FixedSeaHash::seed_from_u64(seed));
        let lineage_store = IndependentLineageStore::default();
        let emigration_exit = NeverEmigrationExit::default();
        let coalescence_sampler = IndependentCoalescenceSampler::default();
        let event_sampler = IndependentEventSampler::default();
        let immigration_entry = NeverImmigrationEntry::default();

        let active_lineage_sampler =
            IndependentActiveLineageSampler::empty(ExpEventTimeSampler::new(args.delta_t.get()));

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

        let block_size = BlockSize::x(args.block_size);
        let grid_size = GridSize::x(args.grid_size);

        with_initialised_cuda(|| {
            let stream = CudaDropWrapper::from(Stream::new(StreamFlags::NON_BLOCKING, None)?);

            SimulationKernel::with_kernel(args.ptx_jit, |kernel| {
                info::print_kernel_function_attributes(kernel.function());

                // TODO: It seems to be more performant to spawn smaller tasks than to use
                //        the full parallelism - why?
                //       Does it have to do with detecting duplication slower (we could increase
                //        the step size bit by bit) or with bottlenecks on the GPU?
                #[allow(clippy::cast_sign_loss)]
                let _max_threads_per_block = kernel
                    .function()
                    .get_attribute(FunctionAttribute::MaxThreadsPerBlock)?
                    as u32;

                let task_list = ValueBuffer::new(&block_size, &grid_size)?;
                let min_spec_sample_buffer = ValueBuffer::new(&block_size, &grid_size)?;

                #[allow(clippy::type_complexity)]
                let event_buffer: EventBuffer<
                    <<P as LocalPartition<R>>::Reporter as Reporter>::ReportSpeciation,
                    <<P as LocalPartition<R>>::Reporter as Reporter>::ReportDispersal,
                > = EventBuffer::new(&block_size, &grid_size, args.step_slice.get().try_into()?)?;

                simulate(
                    &stream,
                    kernel,
                    (grid_size, block_size, args.dedup_cache),
                    simulation,
                    lineages,
                    task_list,
                    event_buffer,
                    min_spec_sample_buffer,
                    local_partition.get_reporter(),
                    args.step_slice,
                )
            })
        })
    }
}
