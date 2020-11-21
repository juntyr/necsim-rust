use necsim_core::{
    cogs::{
        CoalescenceSampler, DispersalSampler, EventSampler, HabitatToU64Injection,
        IncoherentLineageStore, LineageReference, PrimeableRng, SingularActiveLineageSampler,
    },
    simulation::Simulation,
};

use necsim_impls_cuda::{
    event_buffer::common::EventBufferCudaRepresentation,
    task_list::common::TaskListCudaRepresentation,
};

use rustacuda::error::CudaResult;
use rustacuda_core::{DeviceCopy, DevicePointer};

use rust_cuda::common::RustToCuda;

use super::SimulationKernel;

mod with_dimensions;
mod with_stream;

use with_dimensions::SimulationKernelWithDimensions;
use with_stream::SimulationKernelWithDimensionsStream;

impl<
        'k,
        's,
        H: HabitatToU64Injection + RustToCuda,
        G: PrimeableRng<H> + RustToCuda,
        D: DispersalSampler<H, G> + RustToCuda,
        R: LineageReference<H> + DeviceCopy,
        S: IncoherentLineageStore<H, R> + RustToCuda,
        C: CoalescenceSampler<H, G, R, S> + RustToCuda,
        E: EventSampler<H, G, D, R, S, C> + RustToCuda,
        A: SingularActiveLineageSampler<H, G, D, R, S, C, E> + RustToCuda,
        const REPORT_SPECIATION: bool,
        const REPORT_DISPERSAL: bool,
    >
    SimulationKernelWithDimensionsStream<
        'k,
        's,
        H,
        G,
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
        simulation_ptr: DevicePointer<
            <Simulation<H, G, D, R, S, C, E, A> as RustToCuda>::CudaRepresentation,
        >,
        task_list_ptr: DevicePointer<TaskListCudaRepresentation<H, R>>,
        event_buffer_ptr: DevicePointer<
            EventBufferCudaRepresentation<H, R, REPORT_SPECIATION, REPORT_DISPERSAL>,
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
            >>>(simulation_ptr, task_list_ptr, event_buffer_ptr, max_steps)
        )
    }
}
