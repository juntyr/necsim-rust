use alloc::boxed::Box;
use core::marker::PhantomData;

use rustacuda_core::DeviceCopy;

use rust_cuda::common::RustToCuda;

use necsim_core::{
    cogs::{Habitat, LineageReference},
    event::{Event, EventType},
    reporter::{EventFilter, Reporter},
};

#[allow(clippy::module_name_repetitions)]
pub struct EventBufferDevice<
    H: Habitat + RustToCuda,
    R: LineageReference<H> + DeviceCopy,
    P: Reporter<H, R>,
> {
    event_counter: usize,
    event_buffer: Box<[Option<Event<H, R>>]>,
    marker: PhantomData<P>,
}

impl<H: Habitat + RustToCuda, R: LineageReference<H> + DeviceCopy, P: Reporter<H, R>> EventFilter
    for EventBufferDevice<H, R, P>
{
    const REPORT_DISPERSAL: bool = P::REPORT_DISPERSAL;
    const REPORT_SPECIATION: bool = P::REPORT_SPECIATION;
}

impl<H: Habitat + RustToCuda, R: LineageReference<H> + DeviceCopy, P: Reporter<H, R>> Reporter<H, R>
    for EventBufferDevice<H, R, P>
{
    #[debug_requires(
        self.event_counter < self.event_buffer.len(),
        "does not report extraneous events"
    )]
    fn report_event(&mut self, event: &Event<H, R>) {
        if (P::REPORT_SPECIATION && matches!(event.r#type(), EventType::Speciation))
            || (P::REPORT_DISPERSAL && matches!(event.r#type(), EventType::Dispersal {..}))
        {
            self.event_buffer[self.event_counter].replace(event.clone());

            self.event_counter += 1;
        }
    }
}

impl<H: Habitat + RustToCuda, R: LineageReference<H> + DeviceCopy, P: Reporter<H, R>>
    EventBufferDevice<H, R, P>
{
    /// # Safety
    /// This function is only safe to call iff `cuda_repr_ptr` is the
    /// `DevicePointer` borrowed on the CPU using the corresponding
    /// `LendToCuda::lend_to_cuda`.
    pub unsafe fn with_borrow_from_rust_mut<O, F: FnOnce(&mut Self) -> O>(
        cuda_repr_ptr: *mut super::common::EventBufferCudaRepresentation<H, R, P>,
        inner: F,
    ) -> O {
        let cuda_repr_ref: &mut super::common::EventBufferCudaRepresentation<H, R, P> =
            &mut *cuda_repr_ptr;

        let buffer_len =
            cuda_repr_ref.block_size * cuda_repr_ref.grid_size * cuda_repr_ref.max_events;

        let raw_slice: &mut [Option<Event<H, R>>] =
            core::slice::from_raw_parts_mut(cuda_repr_ref.device_buffer.as_raw_mut(), buffer_len);

        let (_before_raw_slice, rest_raw_slice) = raw_slice
            .split_at_mut(rust_cuda::device::utils::index_no_offset() * cuda_repr_ref.max_events);
        let (individual_raw_slice, _after_raw_slice) =
            rest_raw_slice.split_at_mut(cuda_repr_ref.max_events);

        let mut rust_repr = EventBufferDevice {
            event_counter: 0,
            event_buffer: alloc::boxed::Box::from_raw(individual_raw_slice),
            marker: cuda_repr_ref.marker,
        };

        let result = inner(&mut rust_repr);

        // MUST forget about rust_repr as we do NOT own any of the heap memory
        // it might reference
        core::mem::forget(rust_repr);

        result
    }
}
