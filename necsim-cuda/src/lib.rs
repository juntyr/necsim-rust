#![deny(clippy::pedantic)]

use std::collections::HashSet;
use std::ffi::CString;

#[macro_use]
extern crate contracts;

#[macro_use]
extern crate rustacuda;

use anyhow::Result;
use array2d::Array2D;

use rustacuda::context::Context as CudaContext;
use rustacuda::function::Function;
use rustacuda::module::Symbol;
use rustacuda::prelude::*;

use rust_cuda::host::LendToCuda;

use necsim_core::cogs::{LineageStore, PrimeableRng};
use necsim_core::reporter::Reporter;
use necsim_core::simulation::Simulation;

use necsim_impls_cuda::cogs::rng::CudaRng;
use necsim_impls_cuda::event_buffer::host::EventBufferHost;
use necsim_impls_no_std::cogs::active_lineage_sampler::independent::IndependentActiveLineageSampler;
use necsim_impls_no_std::cogs::coalescence_sampler::independent::IndependentCoalescenceSampler;
use necsim_impls_no_std::cogs::dispersal_sampler::in_memory::packed_alias::InMemoryPackedAliasDispersalSampler;
use necsim_impls_no_std::cogs::event_sampler::independent::IndependentEventSampler;
use necsim_impls_no_std::cogs::habitat::in_memory::InMemoryHabitat;
use necsim_impls_no_std::cogs::lineage_reference::in_memory::InMemoryLineageReference;
use necsim_impls_no_std::cogs::lineage_store::incoherent::in_memory::IncoherentInMemoryLineageStore;
use necsim_impls_std::cogs::dispersal_sampler::in_memory::InMemoryDispersalSampler;

macro_rules! with_cuda {
    ($init:expr => |$var:ident: $r#type:ty| $inner:block) => {
        let $var = $init;

        $inner

        if let Err((_err, val)) = <$r#type>::drop($var) {
            std::mem::forget(val);
        }
    };
    ($init:expr => |mut $var:ident: $r#type:ty| $inner:block) => {
        let mut $var = $init;

        $inner

        if let Err((_err, val)) = <$r#type>::drop($var) {
            std::mem::forget(val);
        }
    };
}

fn print_context_resource_limits() {
    use rustacuda::context::{CurrentContext, ResourceLimit};

    println!("{:=^80}", " Context Resource Limits ");

    println!(
        "StackSize: {:?}",
        CurrentContext::get_resource_limit(ResourceLimit::StackSize)
    );
    println!(
        "PrintfFifoSize: {:?}",
        CurrentContext::get_resource_limit(ResourceLimit::PrintfFifoSize)
    );
    println!(
        "MallocHeapSize: {:?}",
        CurrentContext::get_resource_limit(ResourceLimit::MallocHeapSize)
    );
    println!(
        "DeviceRuntimeSynchronizeDepth: {:?}",
        CurrentContext::get_resource_limit(ResourceLimit::DeviceRuntimeSynchronizeDepth)
    );
    println!(
        "DeviceRuntimePendingLaunchCount: {:?}",
        CurrentContext::get_resource_limit(ResourceLimit::DeviceRuntimePendingLaunchCount)
    );
    println!(
        "MaxL2FetchGranularity: {:?}",
        CurrentContext::get_resource_limit(ResourceLimit::MaxL2FetchGranularity)
    );

    println!("{:=^80}", "");
}

fn print_kernel_function_attributes(kernel: &Function) {
    use rustacuda::function::FunctionAttribute;

    println!("{:=^80}", " Kernel Function Attributes ");

    println!(
        "MaxThreadsPerBlock: {:?}",
        kernel.get_attribute(FunctionAttribute::MaxThreadsPerBlock)
    );
    println!(
        "SharedMemorySizeBytes: {:?}",
        kernel.get_attribute(FunctionAttribute::SharedMemorySizeBytes)
    );
    println!(
        "ConstSizeBytes: {:?}",
        kernel.get_attribute(FunctionAttribute::ConstSizeBytes)
    );
    println!(
        "LocalSizeBytes: {:?}",
        kernel.get_attribute(FunctionAttribute::LocalSizeBytes)
    );
    println!(
        "NumRegisters: {:?}",
        kernel.get_attribute(FunctionAttribute::NumRegisters)
    );
    println!(
        "PtxVersion: {:?}",
        kernel.get_attribute(FunctionAttribute::PtxVersion)
    );
    println!(
        "BinaryVersion: {:?}",
        kernel.get_attribute(FunctionAttribute::BinaryVersion)
    );
    println!(
        "CacheModeCa: {:?}",
        kernel.get_attribute(FunctionAttribute::CacheModeCa)
    );

    println!("{:=^80}", "");
}

pub struct CudaSimulation;

impl CudaSimulation {
    /// Simulates the coalescence algorithm on a CUDA-capable GPU on an in
    /// memory `habitat` with precalculated `dispersal`.
    ///
    /// # Errors
    ///
    /// `Err(InconsistentDispersalMapSize)` is returned iff the dimensions of
    /// `dispersal` are not `ExE` given `E=RxC` where `habitat` has dimension
    /// `RxC`.
    #[debug_requires(
        speciation_probability_per_generation >= 0.0_f64 &&
        speciation_probability_per_generation <= 1.0_f64,
        "0.0 <= speciation_probability_per_generation <= 1.0"
    )]
    #[debug_requires(
        sample_percentage >= 0.0_f64 &&
        sample_percentage <= 1.0_f64,
        "0.0 <= sample_percentage <= 1.0"
    )]
    pub fn simulate<G: PrimeableRng<InMemoryHabitat>>(
        habitat: &Array2D<u32>,
        dispersal: &Array2D<f64>,
        speciation_probability_per_generation: f64,
        sample_percentage: f64,
        rng: G,
        reporter: &mut impl Reporter<InMemoryHabitat, InMemoryLineageReference>,
    ) -> Result<(f64, u64)> {
        const SIMULATION_STEP_SLICE: usize = 100_usize;

        let habitat = InMemoryHabitat::new(habitat.clone());
        let dispersal_sampler = InMemoryPackedAliasDispersalSampler::new(dispersal, &habitat)?;
        let lineage_store = IncoherentInMemoryLineageStore::new(sample_percentage, &habitat);
        let coalescence_sampler = IndependentCoalescenceSampler::default();
        let event_sampler = IndependentEventSampler::default();
        let active_lineage_sampler = IndependentActiveLineageSampler::default();

        let mut simulation = Simulation::builder()
            .speciation_probability_per_generation(speciation_probability_per_generation)
            .habitat(habitat)
            .rng(CudaRng::from(rng))
            .dispersal_sampler(dispersal_sampler)
            .lineage_reference(std::marker::PhantomData::<InMemoryLineageReference>)
            .lineage_store(lineage_store)
            .coalescence_sampler(coalescence_sampler)
            .event_sampler(event_sampler)
            .active_lineage_sampler(active_lineage_sampler)
            .build();

        // TODO: Need a way to tune these based on the available CUDA device or cmd args
        let cuda_block_size = rustacuda::function::BlockSize::xy(16, 16);
        let cuda_grid_size = rustacuda::function::GridSize::xy(16, 16);

        #[allow(clippy::cast_possible_truncation)]
        let cuda_grid_amount = {
            #[allow(clippy::cast_possible_truncation)]
            let total_individuals = simulation.lineage_store().get_number_total_lineages();

            let cuda_block_size =
                (cuda_block_size.x * cuda_block_size.y * cuda_block_size.z) as usize;
            let cuda_grid_size = (cuda_grid_size.x * cuda_grid_size.y * cuda_grid_size.z) as usize;

            let cuda_task_size = cuda_block_size * cuda_grid_size;

            (total_individuals / cuda_task_size) + (total_individuals % cuda_task_size > 0) as usize
        } as u32;

        let module_data = CString::new(include_str!(env!("KERNEL_PTX_PATH"))).unwrap();

        // unimplemented!("{}", include_str!(env!("KERNEL_PTX_PATH")));

        // Initialize the CUDA API
        rustacuda::init(CudaFlags::empty())?;

        // Get the first device
        let device = Device::get_device(0)?;

        let mut global_time_max = 0.0_f64;
        let mut global_steps_sum = 0_u64;

        let mut event_deduplicator = HashSet::new();

        // Create a context associated to this device
        with_cuda!(CudaContext::create_and_push(ContextFlags::MAP_HOST | ContextFlags::SCHED_AUTO, device)? => |context: CudaContext| {
        // Load the module containing the kernel function
        with_cuda!(Module::load_from_string(&module_data)? => |module: Module| {
        // Load and initialise the grid_id symbol from the module
        let mut grid_id_symbol: Symbol<u32>  = module.get_global(&CString::new("grid_id").unwrap())?;
        grid_id_symbol.copy_from(&0_u32)?;
        // Load the kernel function from the module
        let simulate_kernel = module.get_function(&CString::new("simulate").unwrap())?;
        // Create a stream to submit work to
        with_cuda!(Stream::new(StreamFlags::NON_BLOCKING, None)? => |stream: Stream| {

            use rustacuda::context::{CurrentContext, ResourceLimit};

            CurrentContext::set_resource_limit(ResourceLimit::StackSize, 4096)?;

            print_context_resource_limits();
            print_kernel_function_attributes(&simulate_kernel);

            let mut event_buffer: EventBufferHost<InMemoryHabitat, InMemoryLineageReference, false, false> =
                EventBufferHost::new(&cuda_block_size, &cuda_grid_size, SIMULATION_STEP_SLICE)?;

            // Load and initialise the global_lineages_remaining symbol
            let mut global_lineages_remaining = simulation.lineage_store().get_number_total_lineages() as u64;
            let mut global_lineages_remaining_symbol: Symbol<u64> =
                module.get_global(&CString::new("global_lineages_remaining").unwrap())?;
            global_lineages_remaining_symbol.copy_from(&global_lineages_remaining)?;

            // Load and initialise the global_time_max and global_steps_sum symbols
            let mut global_time_max_symbol: Symbol<f64> = module.get_global(&CString::new("global_time_max").unwrap())?;
            global_time_max_symbol.copy_from(&0.0_f64)?;
            let mut global_steps_sum_symbol: Symbol<u64> = module.get_global(&CString::new("global_steps_sum").unwrap())?;
            global_steps_sum_symbol.copy_from(&0_u64)?;

            // TODO: We should use async launches and callbacks to rotate between simulation, event analysis etc.
            if let Err(err) = simulation.lend_to_cuda_mut(|simulation_mut_ptr| {
                let mut time_slice = 0;

                while global_lineages_remaining > 0 {
                    println!("Starting time slice {} with {} remaining individuals ...", time_slice + 1, global_lineages_remaining);

                    for grid_id in 0..cuda_grid_amount {
                        grid_id_symbol.copy_from(&grid_id)?;

                        let cuda_grid_size = cuda_grid_size.clone();
                        let cuda_block_size = cuda_block_size.clone();

                        println!("Launching grid {}/{} of time slice {} ...", grid_id + 1, cuda_grid_amount, time_slice + 1);

                        // Launching kernels is unsafe since Rust cannot enforce safety across
                        // the foreign function CUDA-C language barrier
                        unsafe {
                            launch!(simulate_kernel<<<cuda_grid_size, cuda_block_size, 0, stream>>>(
                                simulation_mut_ptr,
                                event_buffer.get_mut_cuda_ptr(),
                                SIMULATION_STEP_SLICE as u64
                            ))?;
                        }

                        println!("Synchronising ...");

                        stream.synchronize()?;

                        global_lineages_remaining_symbol.copy_to(&mut global_lineages_remaining)?;

                        println!("Analysing events ...");

                        event_buffer.with_fetched_events(|events| {
                            events.filter(|event| {
                                event_deduplicator.insert(event.clone())
                            }).for_each(|event| reporter.report_event(&event))
                        })?
                    }

                    time_slice += 1;
                }

                Ok(())
            }) {
                eprintln!("\nRunning kernel failed with {:#?}!\n", err);
            }

            global_time_max_symbol.copy_to(&mut global_time_max)?;
            global_steps_sum_symbol.copy_to(&mut global_steps_sum)?;

        });});});

        //println!("{:#?}", event_deduplicator);

        Ok((global_time_max, global_steps_sum))
    }
}
