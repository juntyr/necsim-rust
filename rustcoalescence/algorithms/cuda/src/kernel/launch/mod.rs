use std::ffi::CString;

use necsim_core::{
    cogs::{
        CoalescenceSampler, DispersalSampler, EmigrationExit, Habitat, ImmigrationEntry,
        LineageReference, LineageStore, MinSpeciationTrackingEventSampler, PrimeableRng,
        SingularActiveLineageSampler, SpeciationProbability, SpeciationSample, TurnoverRate,
    },
    lineage::Lineage,
    reporter::boolean::Boolean,
    simulation::Simulation,
};

use necsim_impls_cuda::{event_buffer::EventBuffer, value_buffer::ValueBuffer};

use rustacuda::{error::CudaResult, function::Function, memory::DeviceBox, module::Module};
use rustacuda_core::DeviceCopy;

use rust_cuda::{
    common::{DeviceBoxMut, RustToCuda},
    host::{CudaDropWrapper, HostDeviceBoxMut},
};
use rustcoalescence_algorithms_cuda_kernel_ptx_jit::{
    compilePtxJITwithArguments, host::compiler::PtxJITResult,
};

use super::SimulationKernel;

mod with_dimensions;
mod with_stream;

use with_dimensions::SimulationKernelWithDimensions;
use with_stream::SimulationKernelWithDimensionsStream;

impl<
        'k,
        's,
        H: Habitat + RustToCuda,
        G: PrimeableRng + RustToCuda,
        R: LineageReference<H> + DeviceCopy,
        S: LineageStore<H, R> + RustToCuda,
        X: EmigrationExit<H, G, R, S> + RustToCuda,
        D: DispersalSampler<H, G> + RustToCuda,
        C: CoalescenceSampler<H, R, S> + RustToCuda,
        T: TurnoverRate<H> + RustToCuda,
        N: SpeciationProbability<H> + RustToCuda,
        E: MinSpeciationTrackingEventSampler<H, G, R, S, X, D, C, T, N> + RustToCuda,
        I: ImmigrationEntry + RustToCuda,
        A: SingularActiveLineageSampler<H, G, R, S, X, D, C, T, N, E, I> + RustToCuda,
        ReportSpeciation: Boolean,
        ReportDispersal: Boolean,
    >
    SimulationKernelWithDimensionsStream<
        'k,
        's,
        H,
        G,
        R,
        S,
        X,
        D,
        C,
        T,
        N,
        E,
        I,
        A,
        ReportSpeciation,
        ReportDispersal,
    >
{
    #[allow(clippy::type_complexity, clippy::too_many_arguments)]
    pub unsafe fn launch_and_synchronise(
        &mut self,
        simulation_ptr: &mut HostDeviceBoxMut<
            <Simulation<H, G, R, S, X, D, C, T, N, E, I, A> as RustToCuda>::CudaRepresentation,
        >,
        task_list_ptr: &mut HostDeviceBoxMut<
            <ValueBuffer<Lineage> as RustToCuda>::CudaRepresentation,
        >,
        event_buffer_ptr: &mut HostDeviceBoxMut<
            <EventBuffer<ReportSpeciation, ReportDispersal> as RustToCuda>::CudaRepresentation,
        >,
        min_spec_sample_buffer_ptr: &mut HostDeviceBoxMut<
            <ValueBuffer<SpeciationSample> as RustToCuda>::CudaRepresentation,
        >,
        total_time_max: &mut DeviceBox<u64>,
        total_steps_sum: &mut DeviceBox<u64>,
        max_steps: u64,
    ) -> CudaResult<()> {
        if self.ptx_jit {
            let compiler = &mut *self.compiler;

            if let PtxJITResult::Recomputed(ptx_cstr) = compilePtxJITwithArguments! {
                compiler(
                    ConstLoad[simulation_ptr.for_host()],
                    ConstLoad[task_list_ptr.for_host()],
                    ConstLoad[event_buffer_ptr.for_host()],
                    ConstLoad[min_spec_sample_buffer_ptr.for_host()],
                    Ignore[total_time_max],
                    Ignore[total_steps_sum],
                    Ignore[max_steps]
                )
            } {
                // JIT compile the CUDA module with the updated PTX string
                let module = Module::load_from_string(ptx_cstr)?;

                // Load the kernel function from the module
                let entry_point = module.get_function(&CString::new("simulate").unwrap())?;

                crate::info::print_kernel_function_attributes(&entry_point);

                // Safety: The swap and drop of the old module is only safe because
                //  - `self.entry_point`, which has the lifetime requirement, is swapped and
                //    dropped first (no stale references)
                //  - `self.module` is swapped into the correct lifetime afterwards
                std::mem::drop(std::mem::replace(
                    self.entry_point,
                    std::mem::transmute::<_, Function<'k>>(entry_point),
                ));
                std::mem::drop(CudaDropWrapper::from(std::mem::replace(
                    self.module,
                    module,
                )));
            }
        }

        let kernel = &self.entry_point;
        let stream = self.stream;

        rustacuda::launch!(
            kernel<<<
                self.grid_size.clone(),
                self.block_size.clone(),
                self.shared_mem_bytes,
                stream
            >>>(
                simulation_ptr.for_device(), task_list_ptr.for_device(),
                event_buffer_ptr.for_device(), min_spec_sample_buffer_ptr.for_device(),
                DeviceBoxMut::from(total_time_max), DeviceBoxMut::from(total_steps_sum), max_steps
            )
        )?;

        stream.synchronize()
    }
}
