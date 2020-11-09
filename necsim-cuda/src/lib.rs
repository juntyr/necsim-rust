#![deny(clippy::pedantic)]

use std::ffi::CString;

#[macro_use]
extern crate contracts;

#[macro_use]
extern crate rustacuda;

use anyhow::Result;
use array2d::Array2D;

use rustacuda::context::Context as CudaContext;
use rustacuda::function::Function;
use rustacuda::prelude::*;

use rust_cuda::host::LendToCuda;

use necsim_core::cogs::{LineageStore, PrimeableRng};
use necsim_core::reporter::Reporter;
use necsim_core::simulation::Simulation;

use necsim_impls_no_std::cogs::active_lineage_sampler::independent::IndependentActiveLineageSampler;
use necsim_impls_no_std::cogs::coalescence_sampler::independent::IndependentCoalescenceSampler;
use necsim_impls_no_std::cogs::dispersal_sampler::in_memory::packed_alias::InMemoryPackedAliasDispersalSampler;
use necsim_impls_no_std::cogs::event_sampler::independent::IndependentEventSampler;
use necsim_impls_no_std::cogs::habitat::in_memory::InMemoryHabitat;
use necsim_impls_no_std::cogs::lineage_reference::in_memory::InMemoryLineageReference;
use necsim_impls_no_std::cogs::lineage_store::incoherent::in_memory::IncoherentInMemoryLineageStore;
use necsim_impls_no_std::cogs::rng::cuda::CudaRng;
use necsim_impls_std::cogs::dispersal_sampler::in_memory::InMemoryDispersalSampler;

macro_rules! with_cuda {
    ($init:expr => |$var:ident: $r#type:ty| $inner:block) => {
        let $var = $init;

        $inner

        if let Err((_err, val)) = <$r#type>::drop($var) {
            //eprintln!("{:?}", err);
            std::mem::forget(val);
        }
    };
    ($init:expr => |mut $var:ident: $r#type:ty| $inner:block) => {
        let mut $var = $init;

        $inner

        if let Err((_err, val)) = <$r#type>::drop($var) {
            //eprintln!("{:?}", err);
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
    pub fn simulate<G: PrimeableRng<Prime = [u8; 16]>>(
        habitat: &Array2D<u32>,
        dispersal: &Array2D<f64>,
        speciation_probability_per_generation: f64,
        sample_percentage: f64,
        rng: G,
        _reporter: &mut impl Reporter<InMemoryHabitat, InMemoryLineageReference>,
    ) -> Result<(f64, usize)> {
        let habitat = InMemoryHabitat::new(habitat.clone());
        let dispersal_sampler = InMemoryPackedAliasDispersalSampler::new(dispersal, &habitat)?;
        let lineage_store = IncoherentInMemoryLineageStore::new(sample_percentage, &habitat);
        let coalescence_sampler = IndependentCoalescenceSampler::default();
        let event_sampler = IndependentEventSampler::default();
        let active_lineage_sampler = IndependentActiveLineageSampler::default();

        // TODO: Should we copy the heap contents back over?
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

        let cuda_block_size = rustacuda::function::BlockSize::xy(16, 16);
        let cuda_grid_size = rustacuda::function::GridSize::x({
            #[allow(clippy::cast_possible_truncation)]
            let total_individuals = simulation.lineage_store().get_number_total_lineages() as u32;
            let cuda_block_length = cuda_block_size.x * cuda_block_size.y * cuda_block_size.z;

            (total_individuals / cuda_block_length)
                + (total_individuals % cuda_block_length > 0) as u32
        });

        //let (time, steps) = simulation.simulate(rng, reporter);

        let module_data = CString::new(include_str!(env!("KERNEL_PTX_PATH"))).unwrap();

        //println!("{}", module_data.to_str().unwrap());

        // Initialize the CUDA API
        rustacuda::init(CudaFlags::empty())?;

        // Get the first device
        let device = Device::get_device(0)?;

        // Create a context associated to this device
        with_cuda!(CudaContext::create_and_push(ContextFlags::MAP_HOST | ContextFlags::SCHED_AUTO, device)? => |context: CudaContext| {
        // Load the module containing the kernel function
        with_cuda!(Module::load_from_string(&module_data)? => |module: Module| {
        // Load the kernel function from the module
        let simulate_kernel = module.get_function(&CString::new("simulate").unwrap())?;
        // Create a stream to submit work to
        with_cuda!(Stream::new(StreamFlags::NON_BLOCKING, None)? => |stream: Stream| {

            use rustacuda::context::{CurrentContext, ResourceLimit};

            CurrentContext::set_resource_limit(ResourceLimit::StackSize, 4096)?;

            print_context_resource_limits();
            print_kernel_function_attributes(&simulate_kernel);

            if let Err(err) = simulation.lend_to_cuda_mut(|simulation_mut_ptr| {
                // Launching kernels is unsafe since Rust can't enforce safety - think of kernel launches
                // as a foreign-function call. In this case, it is - this kernel is written in CUDA C.
                unsafe {
                    launch!(simulate_kernel<<<cuda_grid_size, cuda_block_size, 0, stream>>>(
                        simulation_mut_ptr,
                        1_000_usize // max steps on GPU
                    ))?;
                }

                stream.synchronize()
            }) {
                eprintln!("Running kernel failed with {:#?}!", err);
            }

        });});});

        let (time, steps) = (4.2_f64, 42);

        Ok((time, steps))
    }
}
