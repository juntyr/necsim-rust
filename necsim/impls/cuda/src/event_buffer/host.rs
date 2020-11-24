use core::ops::DerefMut;

use hashbrown::hash_map::{HashMap, RawEntryMut};
type HashSet<K> = HashMap<K, ()>;

use rustacuda::{
    error::CudaResult,
    function::{BlockSize, GridSize},
    memory::{CopyDestination, DeviceBox, DeviceBuffer, LockedBuffer},
};

use rustacuda_core::{DeviceCopy, DevicePointer};

use rust_cuda::common::RustToCuda;

use rust_cuda::host::CudaDropWrapper;

use necsim_core::{
    cogs::{Habitat, LineageReference},
    event::Event,
    reporter::Reporter,
};

#[allow(clippy::module_name_repetitions)]
pub struct EventBufferHost<
    'r,
    H: Habitat + RustToCuda,
    R: LineageReference<H> + DeviceCopy,
    P: Reporter<H, R>,
    const REPORT_SPECIATION: bool,
    const REPORT_DISPERSAL: bool,
> {
    host_buffer: CudaDropWrapper<LockedBuffer<Option<Event<H, R>>>>,
    device_buffer: CudaDropWrapper<DeviceBuffer<Option<Event<H, R>>>>,
    cuda_repr_box: CudaDropWrapper<
        DeviceBox<
            super::common::EventBufferCudaRepresentation<H, R, REPORT_SPECIATION, REPORT_DISPERSAL>,
        >,
    >,
    reporter: &'r mut P,
    event_deduplicator: HashSet<Event<H, R>>,
}

impl<
        'r,
        H: Habitat + RustToCuda,
        R: LineageReference<H> + DeviceCopy,
        P: Reporter<H, R>,
        const REPORT_SPECIATION: bool,
        const REPORT_DISPERSAL: bool,
    > EventBufferHost<'r, H, R, P, REPORT_SPECIATION, REPORT_DISPERSAL>
{
    /// # Errors
    /// Returns a `rustacuda::errors::CudaError` iff an error occurs inside CUDA
    pub fn new(
        reporter: &'r mut P,
        block_size: &BlockSize,
        grid_size: &GridSize,
        max_events: usize,
    ) -> CudaResult<Self> {
        if P::REPORT_SPECIATION != REPORT_SPECIATION || P::REPORT_DISPERSAL != REPORT_DISPERSAL {
            // Rust does not yet allow enforcing associated const bounds at compile time
            unimplemented!(
                "EventBuffer was initialised with mismatching reporting \
                 requirements:\nREPORT_SPECIATION: {} vs {}\nREPORT_DISPERSAL: {} vs {}",
                P::REPORT_SPECIATION,
                REPORT_SPECIATION,
                P::REPORT_DISPERSAL,
                REPORT_DISPERSAL
            );
        }

        let max_events = if P::REPORT_DISPERSAL {
            max_events
        } else if P::REPORT_SPECIATION {
            1_usize
        } else {
            0_usize
        };

        let block_size = (block_size.x * block_size.y * block_size.z) as usize;
        let grid_size = (grid_size.x * grid_size.y * grid_size.z) as usize;
        let total_capacity = max_events * block_size * grid_size;

        let host_buffer = CudaDropWrapper::from(LockedBuffer::new(&None, total_capacity)?);
        let mut device_buffer =
            CudaDropWrapper::from(DeviceBuffer::from_slice(host_buffer.as_slice())?);

        let cuda_repr = super::common::EventBufferCudaRepresentation {
            block_size,
            grid_size,
            max_events,
            device_buffer: device_buffer.as_device_ptr(),
        };

        let cuda_repr_box = CudaDropWrapper::from(DeviceBox::new(&cuda_repr)?);

        Ok(Self {
            host_buffer,
            device_buffer,
            cuda_repr_box,
            reporter,
            event_deduplicator: HashSet::new(),
        })
    }

    /// # Errors
    /// Returns a `rustacuda::errors::CudaError` iff an error occurs inside CUDA
    pub fn fetch_and_report_events(&mut self) -> CudaResult<()> {
        self.device_buffer.copy_to(self.host_buffer.deref_mut())?;

        for event in self.host_buffer.iter_mut().filter_map(Option::take) {
            if let RawEntryMut::Vacant(entry) =
                self.event_deduplicator.raw_entry_mut().from_key(&event)
            {
                self.reporter.report_event(entry.insert(event, ()).0)
            }
        }

        self.device_buffer.copy_from(self.host_buffer.deref_mut())?;

        Ok(())
    }

    pub fn get_mut_cuda_ptr(
        &mut self,
    ) -> DevicePointer<
        super::common::EventBufferCudaRepresentation<H, R, REPORT_SPECIATION, REPORT_DISPERSAL>,
    > {
        self.cuda_repr_box.as_device_ptr()
    }
}
