use core::marker::PhantomData;

use rustacuda_core::{DeviceCopy, DevicePointer};

use rust_cuda::common::RustToCuda;

use necsim_core::{
    cogs::{Habitat, LineageReference},
    event::Event,
    reporter::Reporter,
};

#[allow(clippy::module_name_repetitions)]
#[allow(dead_code)]
pub struct EventBufferCudaRepresentation<
    H: Habitat + RustToCuda,
    R: LineageReference<H> + DeviceCopy,
    P: Reporter<H, R>,
> {
    pub(super) block_size: usize,
    pub(super) grid_size: usize,
    pub(super) max_events: usize,
    pub(super) device_buffer: DevicePointer<Option<Event<H, R>>>,
    pub(super) marker: PhantomData<P>,
}

unsafe impl<H: Habitat + RustToCuda, R: LineageReference<H> + DeviceCopy, P: Reporter<H, R>>
    DeviceCopy for EventBufferCudaRepresentation<H, R, P>
{
}
