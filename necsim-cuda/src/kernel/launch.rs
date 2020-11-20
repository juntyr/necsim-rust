use necsim_core::{
    cogs::{
        ActiveLineageSampler, CoalescenceSampler, DispersalSampler, EventSampler,
        HabitatToU64Injection, IncoherentLineageStore, LineageReference, PrimeableRng,
    },
    simulation::Simulation,
};

use necsim_impls_cuda::event_buffer::common::EventBufferCudaRepresentation;

use rustacuda::{
    error::CudaResult,
    function::{BlockSize, GridSize},
    stream::Stream,
};
use rustacuda_core::{DeviceCopy, DevicePointer};

use rust_cuda::common::RustToCuda;

use super::SimulationKernel;

impl<
        'k,
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
    > SimulationKernel<'k, H, G, D, R, S, C, E, A, REPORT_SPECIATION, REPORT_DISPERSAL>
{
    pub unsafe fn launch<I: Into<GridSize>, B: Into<BlockSize>>(
        &self,
        stream: &Stream,

        // Launch parameters
        grid_size: I,
        block_size: B,
        shared_mem_bytes: u32,

        // Kernel parameters
        simulation_ptr: DevicePointer<
            <Simulation<H, G, D, R, S, C, E, A> as RustToCuda>::CudaRepresentation,
        >,
        event_buffer_ptr: DevicePointer<
            EventBufferCudaRepresentation<H, R, REPORT_SPECIATION, REPORT_DISPERSAL>,
        >,
        max_steps: u64,
    ) -> CudaResult<()> {
        let kernel = self.entry_point;

        rustacuda::launch!(kernel<<<grid_size, block_size, shared_mem_bytes, stream>>>(simulation_ptr, event_buffer_ptr, max_steps))
    }
}

#[macro_export]
macro_rules! launch {
    ($kernel:ident <<<$grid:expr, $block:expr, $shared:expr, $stream:ident>>>($($param:expr),*)) => {
        $kernel.launch($stream, $grid, $block, $shared, $($param),*)
    };
}
