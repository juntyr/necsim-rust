#![deny(clippy::pedantic)]
#![feature(min_const_generics)]

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

mod kernel {
    use std::{
        ffi::{CStr, CString},
        os::raw::c_char,
    };

    // This function should do a switch on the strings and return the correct kernel
    // LTO should be able to optimise the call away
    extern "C" {
        #[no_mangle]
        fn get_ptx_cstr_for_specialisation(specialisation: *const c_char) -> *const c_char;
    }

    use necsim_core::cogs::{
        ActiveLineageSampler, CoalescenceSampler, DispersalSampler, EventSampler,
        HabitatToU64Injection, IncoherentLineageStore, LineageReference, PrimeableRng,
    };
    use rust_cuda::common::RustToCuda;
    use rustacuda_core::DeviceCopy;

    pub fn get_ptx_cstr<
        H: HabitatToU64Injection + RustToCuda,
        G: PrimeableRng<H> + RustToCuda,
        D: DispersalSampler<H, G> + RustToCuda,
        R: LineageReference<H> + DeviceCopy,
        S: IncoherentLineageStore<H, R> + RustToCuda,
        C: CoalescenceSampler<H, G, R, S> + RustToCuda,
        E: EventSampler<H, G, D, R, S, C> + RustToCuda,
        A: ActiveLineageSampler<H, G, D, R, S, C, E> + RustToCuda,
        const REPORT_SPECIATION: bool,
        const REPORT_DISPERSAL: bool,
    >() -> &'static CStr {
        fn type_name_of<T>(_: T) -> CString {
            CString::new(std::any::type_name::<T>()).unwrap()
        }

        let type_name_cstring = type_name_of(
            get_ptx_cstr::<H, G, D, R, S, C, E, A, REPORT_SPECIATION, REPORT_DISPERSAL>,
        );

        let ptx_c_chars = unsafe { get_ptx_cstr_for_specialisation(type_name_cstring.as_ptr()) };

        unsafe { CStr::from_ptr(ptx_c_chars as *const i8) }
    }
}

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

            let (time, steps) = with_cuda_kernel(
                kernel::get_ptx_cstr::<
                    config::Habitat,
                    config::Rng,
                    config::DispersalSampler,
                    config::LineageReference,
                    config::LineageStore,
                    config::CoalescenceSampler,
                    config::EventSampler,
                    config::ActiveLineageSampler,
                    { config::REPORT_SPECIATION },
                    { config::REPORT_DISPERSAL },
                >(),
                |stream, module, kernel| {
                    #[allow(clippy::type_complexity)]
                    let event_buffer: EventBufferHost<
                        config::Habitat,
                        config::LineageReference,
                        P::Reporter<config::Habitat, config::LineageReference>,
                        { config::REPORT_SPECIATION },
                        { config::REPORT_DISPERSAL },
                    > = EventBufferHost::new(
                        reporter,
                        &block_size,
                        &grid_size,
                        SIMULATION_STEP_SLICE,
                    )?;

                    simulate(
                        simulation,
                        SIMULATION_STEP_SLICE as u64,
                        event_buffer,
                        &stream,
                        &module,
                        &kernel,
                        (grid_amount, grid_size, block_size),
                    )
                },
            )?;

            Ok((time, steps))
        })
    }
}
