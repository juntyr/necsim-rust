use rustacuda_core::{DeviceCopy, DevicePointer};

use rust_cuda::common::RustToCuda;

use necsim_core::{
    cogs::{Habitat, LineageReference},
    event::Event,
};

#[allow(clippy::module_name_repetitions)]
#[allow(dead_code)]
pub struct EventBufferCudaRepresentation<
    H: Habitat + RustToCuda,
    R: LineageReference<H> + DeviceCopy,
    const REPORT_SPECIATION: bool,
    const REPORT_DISPERSAL: bool,
> {
    pub(super) block_size: usize,
    pub(super) grid_size: usize,
    pub(super) max_events: usize,
    pub(super) device_buffer: DevicePointer<Option<Event<H, R>>>,
}

unsafe impl<
        H: Habitat + RustToCuda,
        R: LineageReference<H> + DeviceCopy,
        const REPORT_SPECIATION: bool,
        const REPORT_DISPERSAL: bool,
    > DeviceCopy for EventBufferCudaRepresentation<H, R, REPORT_SPECIATION, REPORT_DISPERSAL>
{
}
