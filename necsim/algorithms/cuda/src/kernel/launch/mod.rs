use necsim_core::{
    cogs::{
        CoalescenceSampler, DispersalSampler, Habitat, IncoherentLineageStore, LineageReference,
        MinSpeciationTrackingEventSampler, PrimeableRng, SingularActiveLineageSampler,
        SpeciationProbability, SpeciationSample,
    },
    simulation::Simulation,
};

use necsim_impls_cuda::{event_buffer::EventBuffer, value_buffer::ValueBuffer};

use rustacuda::error::CudaResult;
use rustacuda_core::DeviceCopy;

use rust_cuda::common::{DeviceBoxMut, RustToCuda};

use super::SimulationKernel;

mod with_dimensions;
mod with_stream;

use with_dimensions::SimulationKernelWithDimensions;
use with_stream::SimulationKernelWithDimensionsStream;

impl<
        'k,
        's,
        H: Habitat + RustToCuda,
        G: PrimeableRng<H> + RustToCuda,
        N: SpeciationProbability<H> + RustToCuda,
        D: DispersalSampler<H, G> + RustToCuda,
        R: LineageReference<H> + DeviceCopy,
        S: IncoherentLineageStore<H, R> + RustToCuda,
        C: CoalescenceSampler<H, G, R, S> + RustToCuda,
        E: MinSpeciationTrackingEventSampler<H, G, N, D, R, S, C> + RustToCuda,
        A: SingularActiveLineageSampler<H, G, N, D, R, S, C, E> + RustToCuda,
        const REPORT_SPECIATION: bool,
        const REPORT_DISPERSAL: bool,
    >
    SimulationKernelWithDimensionsStream<
        'k,
        's,
        H,
        G,
        N,
        D,
        R,
        S,
        C,
        E,
        A,
        REPORT_SPECIATION,
        REPORT_DISPERSAL,
    >
{
    #[allow(clippy::type_complexity)]
    pub unsafe fn launch(
        &self,
        simulation_ptr: DeviceBoxMut<
            <Simulation<H, G, N, D, R, S, C, E, A> as RustToCuda>::CudaRepresentation,
        >,
        task_list_ptr: DeviceBoxMut<<ValueBuffer<R> as RustToCuda>::CudaRepresentation>,
        event_buffer_ptr: DeviceBoxMut<
            <EventBuffer<REPORT_SPECIATION, REPORT_DISPERSAL> as RustToCuda>::CudaRepresentation,
        >,
        min_spec_sample_buffer_ptr: DeviceBoxMut<
            <ValueBuffer<SpeciationSample> as RustToCuda>::CudaRepresentation,
        >,
        max_steps: u64,
    ) -> CudaResult<()> {
        let kernel = self.entry_point;
        let stream = self.stream;

        rustacuda::launch!(
            kernel<<<
                self.grid_size.clone(),
                self.block_size.clone(),
                self.shared_mem_bytes,
                stream
            >>>(simulation_ptr, task_list_ptr, event_buffer_ptr, min_spec_sample_buffer_ptr, max_steps)
        )
    }
}
