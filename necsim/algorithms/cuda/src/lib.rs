#![deny(clippy::pedantic)]
#![feature(min_const_generics)]

#[macro_use]
extern crate contracts;

#[macro_use]
extern crate bitvec;

use anyhow::Result;

use rustacuda::{
    function::{BlockSize, FunctionAttribute, GridSize},
    stream::{Stream, StreamFlags},
};

use necsim_core::{
    cogs::{DispersalSampler, Habitat, RngCore},
    lineage::{GlobalLineageReference, Lineage},
    simulation::Simulation,
};

use necsim_impls_cuda::{event_buffer::EventBuffer, value_buffer::ValueBuffer};
use necsim_impls_no_std::reporter::ReporterContext;

use necsim_impls_no_std::cogs::{
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
};

use necsim_impls_cuda::cogs::rng::CudaRng;

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

pub struct CudaArguments {
    pub ptx_jit: bool,
    pub delta_t: f64,
    pub block_size: u32,
    pub grid_size: u32,
    pub step_slice: usize,
}

pub struct CudaSimulation;

impl CudaSimulation {
    /// Simulates the coalescence algorithm on a CUDA-capable GPU on
    /// `habitat` with `dispersal` and lineages from `lineage_store`.
    fn simulate<
        H: Habitat + RustToCuda,
        D: DispersalSampler<H, CudaRng<Rng>> + RustToCuda,
        P: ReporterContext,
    >(
        habitat: H,
        dispersal_sampler: D,
        lineages: Vec<Lineage>,
        speciation_probability_per_generation: f64,
        seed: u64,
        reporter_context: P,
        auxiliary: &CudaArguments,
    ) -> Result<(f64, u64)> {
        const REPORT_SPECIATION: bool = true;
        const REPORT_DISPERSAL: bool = false;

        anyhow::ensure!(
            auxiliary.delta_t > 0.0_f64,
            "CUDA algorithm delta_t must be positive."
        );

        reporter_context.with_reporter(|reporter| {
            let rng = CudaRng::<Rng>::seed_from_u64(seed);
            let speciation_probability =
                UniformSpeciationProbability::new(speciation_probability_per_generation);
            let lineage_store = IndependentLineageStore::default();
            let emigration_exit = NeverEmigrationExit::default();
            let coalescence_sampler = CoalescenceSampler::default();
            let event_sampler = EventSampler::default();
            let immigration_entry = NeverImmigrationEntry::default();

            // TODO: Need to test dt on a variety of seeds to see which is optimal
            let active_lineage_sampler =
                ActiveLineageSampler::empty(ExpEventTimeSampler::new(auxiliary.delta_t));

            let simulation = Simulation::builder()
                .habitat(habitat)
                .rng(rng)
                .speciation_probability(speciation_probability)
                .dispersal_sampler(dispersal_sampler)
                .lineage_reference(std::marker::PhantomData::<GlobalLineageReference>)
                .lineage_store(lineage_store)
                .emigration_exit(emigration_exit)
                .coalescence_sampler(coalescence_sampler)
                .event_sampler(event_sampler)
                .immigration_entry(immigration_entry)
                .active_lineage_sampler(active_lineage_sampler)
                .build();

            let block_size = BlockSize::x(auxiliary.block_size);
            let grid_size = GridSize::x(auxiliary.grid_size);

            let (time, steps) = with_initialised_cuda(|| {
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
                        { REPORT_SPECIATION },
                        { REPORT_DISPERSAL },
                    > = EventBuffer::new(&block_size, &grid_size, auxiliary.step_slice)?;

                    simulate(
                        &stream,
                        kernel,
                        (grid_size, block_size),
                        simulation,
                        lineages.into(),
                        task_list,
                        event_buffer,
                        min_spec_sample_buffer,
                        reporter,
                        auxiliary.step_slice as u64,
                    )
                })
            })?;

            Ok((time, steps))
        })
    }
}
