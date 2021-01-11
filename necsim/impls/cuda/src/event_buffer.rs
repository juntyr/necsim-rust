#[cfg(not(target_os = "cuda"))]
use hashbrown::hash_map::{HashMap, RawEntryMut};

#[cfg(not(target_os = "cuda"))]
type HashSet<K> = HashMap<K, ()>;
#[cfg(target_os = "cuda")]
type HashSet<K> = core::marker::PhantomData<K>;

#[cfg(not(target_os = "cuda"))]
use rustacuda::{
    error::CudaResult,
    function::{BlockSize, GridSize},
};

use rust_cuda::utils::exchange::buffer::CudaExchangeBuffer;

use necsim_core::{event::Event, reporter::Reporter};

#[cfg(target_os = "cuda")]
use necsim_core::event::EventType;
#[cfg(target_os = "cuda")]
use necsim_core::reporter::EventFilter;

#[allow(clippy::module_name_repetitions)]
#[derive(RustToCuda, LendToCuda)]
pub struct EventBuffer<const REPORT_SPECIATION: bool, const REPORT_DISPERSAL: bool> {
    #[r2cEmbed]
    buffer: CudaExchangeBuffer<Option<Event>>,
    #[r2cPhantom(Event)]
    event_deduplicator: HashSet<Event>,
    max_events: usize,
    event_counter: usize,
}

#[cfg(not(target_os = "cuda"))]
impl<const REPORT_SPECIATION: bool, const REPORT_DISPERSAL: bool>
    EventBuffer<REPORT_SPECIATION, REPORT_DISPERSAL>
{
    /// # Errors
    /// Returns a `rustacuda::errors::CudaError` iff an error occurs inside CUDA
    pub fn new(
        block_size: &BlockSize,
        grid_size: &GridSize,
        max_events: usize,
    ) -> CudaResult<Self> {
        let max_events = if REPORT_DISPERSAL {
            max_events
        } else if REPORT_SPECIATION {
            1_usize
        } else {
            0_usize
        };

        let block_size = (block_size.x * block_size.y * block_size.z) as usize;
        let grid_size = (grid_size.x * grid_size.y * grid_size.z) as usize;
        let total_capacity = max_events * block_size * grid_size;

        Ok(Self {
            buffer: CudaExchangeBuffer::new(&None, total_capacity)?,
            event_deduplicator: HashSet::new(),
            max_events,
            event_counter: 0_usize,
        })
    }

    pub fn report_events<P: Reporter>(&mut self, reporter: &mut P) {
        // TODO: Enforce Reporter has the same EventFilter once Rust allows
        //       enforcing associated const bounds at compile time

        for event in self.buffer.iter_mut().filter_map(Option::take) {
            if let RawEntryMut::Vacant(entry) =
                self.event_deduplicator.raw_entry_mut().from_key(&event)
            {
                reporter.report_event(entry.insert(event, ()).0)
            }
        }
    }
}

#[cfg(target_os = "cuda")]
impl<const REPORT_SPECIATION: bool, const REPORT_DISPERSAL: bool> EventFilter
    for EventBuffer<REPORT_SPECIATION, REPORT_DISPERSAL>
{
    const REPORT_DISPERSAL: bool = REPORT_DISPERSAL;
    const REPORT_SPECIATION: bool = REPORT_SPECIATION;
}

#[cfg(target_os = "cuda")]
impl<const REPORT_SPECIATION: bool, const REPORT_DISPERSAL: bool> Reporter
    for EventBuffer<REPORT_SPECIATION, REPORT_DISPERSAL>
{
    #[debug_requires(
        self.event_counter < self.buffer.len(),
        "does not report extraneous events"
    )]
    fn report_event(&mut self, event: &Event) {
        if (REPORT_SPECIATION && matches!(event.r#type(), EventType::Speciation))
            || (REPORT_DISPERSAL && matches!(event.r#type(), EventType::Dispersal {..}))
        {
            self.buffer[rust_cuda::device::utils::index() * self.max_events + self.event_counter]
                .replace(event.clone());

            self.event_counter += 1;
        }
    }
}
