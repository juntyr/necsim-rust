#![deny(clippy::pedantic)]
#![feature(stmt_expr_attributes)]

#[macro_use]
extern crate contracts;

#[macro_use]
extern crate bitvec;

use std::{
    convert::TryInto,
    num::{NonZeroU64, NonZeroUsize},
};

use anyhow::Result;
use serde::Deserialize;

use rustacuda::{
    function::{BlockSize, FunctionAttribute, GridSize},
    stream::{Stream, StreamFlags},
};

use necsim_core::{
    cogs::{DispersalSampler, Habitat, RngCore},
    lineage::{GlobalLineageReference, Lineage},
    reporter::Reporter,
    simulation::Simulation,
};

use necsim_impls_no_std::{
    cogs::{
        active_lineage_sampler::independent::{
            event_time_sampler::exp::ExpEventTimeSampler,
            IndependentActiveLineageSampler as ActiveLineageSampler,
        },
        coalescence_sampler::independent::IndependentCoalescenceSampler as CoalescenceSampler,
        emigration_exit::never::NeverEmigrationExit,
        event_sampler::independent::IndependentEventSampler as EventSampler,
        immigration_entry::never::NeverImmigrationEntry,
        lineage_store::independent::IndependentLineageStore,
        rng::fixedseahash::FixedSeaHash as Rng,
        speciation_probability::uniform::UniformSpeciationProbability,
        turnover_rate::uniform::UniformTurnoverRate,
    },
    partitioning::LocalPartition,
    reporter::ReporterContext,
};

use necsim_impls_std::bounded::PositiveF64;

use necsim_impls_cuda::{cogs::rng::CudaRng, event_buffer::EventBuffer, value_buffer::ValueBuffer};

use rust_cuda::{common::RustToCuda, host::CudaDropWrapper};

mod cuda;
mod info;
mod kernel;
mod simulate;

mod almost_infinite;
mod in_memory;
mod non_spatial;

use crate::kernel::SimulationKernel;
use cuda::with_initialised_cuda;
use simulate::simulate;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AbsoluteDedupCache {
    capacity: NonZeroUsize,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RelativeDedupCache {
    factor: PositiveF64,
}

#[derive(Debug, Deserialize)]
pub enum DedupCache {
    Absolute(AbsoluteDedupCache),
    Relative(RelativeDedupCache),
    None,
}

#[derive(Debug, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct CudaArguments {
    ptx_jit: bool,
    delta_t: PositiveF64,
    block_size: u32,
    grid_size: u32,
    step_slice: NonZeroU64,
    dedup_cache: DedupCache,
}

impl Default for CudaArguments {
    fn default() -> Self {
        Self {
            ptx_jit: false,
            delta_t: PositiveF64::new(1.0_f64).unwrap(),
            block_size: 32_u32,
            grid_size: 256_u32,
            step_slice: NonZeroU64::new(200_u64).unwrap(),
            dedup_cache: DedupCache::Relative(RelativeDedupCache {
                factor: PositiveF64::new(2.0_f64).unwrap(),
            }),
        }
    }
}

pub struct CudaSimulation;

impl CudaSimulation {
    /// Simulates the coalescence algorithm on a CUDA-capable GPU on
    /// `habitat` with `dispersal` and lineages from `lineage_store`.
    fn simulate<
        H: Habitat + RustToCuda,
        D: DispersalSampler<H, CudaRng<Rng>> + RustToCuda,
        R: ReporterContext,
        P: LocalPartition<R>,
    >(
        habitat: H,
        dispersal_sampler: D,
        lineages: Vec<Lineage>,
        speciation_probability_per_generation: f64,
        seed: u64,
        local_partition: &mut P,
        auxiliary: CudaArguments,
    ) -> Result<(f64, u64)> {
        let rng = CudaRng::<Rng>::seed_from_u64(seed);
        let speciation_probability =
            UniformSpeciationProbability::new(speciation_probability_per_generation);
        let lineage_store = IndependentLineageStore::default();
        let emigration_exit = NeverEmigrationExit::default();
        let coalescence_sampler = CoalescenceSampler::default();
        let turnover_rate = UniformTurnoverRate::default();
        let event_sampler = EventSampler::default();
        let immigration_entry = NeverImmigrationEntry::default();

        let active_lineage_sampler =
            ActiveLineageSampler::empty(ExpEventTimeSampler::new(auxiliary.delta_t.get()));

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

        let block_size = BlockSize::x(auxiliary.block_size);
        let grid_size = GridSize::x(auxiliary.grid_size);

        with_initialised_cuda(|| {
            let stream = CudaDropWrapper::from(Stream::new(StreamFlags::NON_BLOCKING, None)?);

            SimulationKernel::with_kernel(auxiliary.ptx_jit, |kernel| {
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
                > = EventBuffer::new(
                    &block_size,
                    &grid_size,
                    auxiliary.step_slice.get().try_into()?,
                )?;

                simulate(
                    &stream,
                    kernel,
                    (grid_size, block_size, auxiliary.dedup_cache),
                    simulation,
                    lineages.into(),
                    task_list,
                    event_buffer,
                    min_spec_sample_buffer,
                    local_partition.get_reporter(),
                    auxiliary.step_slice.get(),
                )
            })
        })
    }
}
