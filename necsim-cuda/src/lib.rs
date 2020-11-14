#![deny(clippy::pedantic)]
#![feature(min_const_generics)]

use std::ffi::CString;

#[macro_use]
extern crate contracts;

#[macro_use]
extern crate rustacuda;

use anyhow::Result;
use array2d::Array2D;

use rustacuda::function::{BlockSize, GridSize};

use necsim_core::{
    cogs::{LineageStore, RngCore},
    simulation::Simulation,
};

use necsim_impls_cuda::event_buffer::host::EventBufferHost;
use necsim_impls_no_std::reporter::ReporterContext;
use necsim_impls_std::{
    cogs::dispersal_sampler::in_memory::InMemoryDispersalSampler,
    simulation::in_memory::InMemorySimulation,
};

mod cuda;
mod info;
mod launch;
mod simulate;

use cuda::with_cuda_kernel;
use simulate::simulate;

pub struct CudaSimulation;

#[contract_trait]
impl InMemorySimulation for CudaSimulation {
    /// Simulates the coalescence algorithm on a CUDA-capable GPU on an in
    /// memory `habitat` with precalculated `dispersal`.
    ///
    /// # Errors
    ///
    /// `Err(InconsistentDispersalMapSize)` is returned iff the dimensions of
    /// `dispersal` are not `ExE` given `E=RxC` where `habitat` has dimension
    /// `RxC`.
    fn simulate<P: ReporterContext>(
        habitat: &Array2D<u32>,
        dispersal: &Array2D<f64>,
        speciation_probability_per_generation: f64,
        sample_percentage: f64,
        seed: u64,
        reporter_context: P,
    ) -> Result<(f64, u64)> {
        const SIMULATION_STEP_SLICE: usize = 100_usize;

        reporter_context.with_reporter(|reporter| {
            let habitat = config::Habitat::new(habitat.clone());
            let rng = config::Rng::seed_from_u64(seed);
            let dispersal_sampler = config::DispersalSampler::new(dispersal, &habitat)?;
            let lineage_store = config::LineageStore::new(sample_percentage, &habitat);
            let coalescence_sampler = config::CoalescenceSampler::default();
            let event_sampler = config::EventSampler::default();
            let active_lineage_sampler = config::ActiveLineageSampler::default();

            let simulation = Simulation::builder()
                .speciation_probability_per_generation(speciation_probability_per_generation)
                .habitat(habitat)
                .rng(rng)
                .dispersal_sampler(dispersal_sampler)
                .lineage_reference(std::marker::PhantomData::<config::LineageReference>)
                .lineage_store(lineage_store)
                .coalescence_sampler(coalescence_sampler)
                .event_sampler(event_sampler)
                .active_lineage_sampler(active_lineage_sampler)
                .build();

            // TODO: Need a way to tune these based on the available CUDA device or cmd args
            let block_size = BlockSize::xy(16, 16);
            let grid_size = GridSize::xy(16, 16);

            #[allow(clippy::cast_possible_truncation)]
            let grid_amount = {
                #[allow(clippy::cast_possible_truncation)]
                let total_individuals = simulation.lineage_store().get_number_total_lineages();

                let block_size = (block_size.x * block_size.y * block_size.z) as usize;
                let grid_size = (grid_size.x * grid_size.y * grid_size.z) as usize;

                let task_size = block_size * grid_size;

                (total_individuals / task_size) + ((total_individuals % task_size > 0) as usize)
            } as u32;

            let module_data = CString::new(include_str!(env!("KERNEL_PTX_PATH"))).unwrap();

            let (time, steps) = with_cuda_kernel(&module_data, |stream, module, kernel| {
                #[allow(clippy::type_complexity)]
                let event_buffer: EventBufferHost<
                    config::Habitat,
                    config::LineageReference,
                    P::Reporter<config::Habitat, config::LineageReference>,
                    { config::REPORT_SPECIATION },
                    { config::REPORT_DISPERSAL },
                > = EventBufferHost::new(reporter, &block_size, &grid_size, SIMULATION_STEP_SLICE)?;

                simulate(
                    simulation,
                    SIMULATION_STEP_SLICE as u64,
                    event_buffer,
                    &stream,
                    &module,
                    &kernel,
                    (grid_amount, grid_size, block_size),
                )
            })?;

            Ok((time, steps))
        })
    }
}
