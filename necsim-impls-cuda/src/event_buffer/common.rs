use rustacuda_core::{DeviceCopy, DevicePointer};

use rust_cuda::common::RustToCuda;

use necsim_core::cogs::{Habitat, LineageReference};
use necsim_core::event::Event;

#[allow(clippy::module_name_repetitions)]
#[allow(dead_code)]
pub struct EventBufferCudaRepresentation<
    H: Habitat + RustToCuda,
    R: LineageReference<H> + DeviceCopy,
> {
    pub(super) block_size: usize,
    pub(super) grid_size: usize,
    pub(super) max_events: usize,
    pub(super) device_buffer: DevicePointer<Option<Event<H, R>>>,
}

unsafe impl<H: Habitat + RustToCuda, R: LineageReference<H> + DeviceCopy> DeviceCopy
    for EventBufferCudaRepresentation<H, R>
{
}
