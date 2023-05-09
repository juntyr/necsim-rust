use core::fmt;

#[cfg(not(target_os = "cuda"))]
use rust_cuda::rustacuda::{
    error::CudaResult,
    function::{BlockSize, GridSize},
};

use rust_cuda::utils::{
    aliasing::SplitSliceOverCudaThreadsDynamicStride, exchange::buffer::CudaExchangeBuffer,
};

use necsim_core::{
    event::{PackedEvent, SpeciationEvent, TypedEvent},
    reporter::{
        boolean::{Boolean, False, True},
        Reporter,
    },
};

#[cfg(target_os = "cuda")]
use necsim_core::impl_report;

use super::utils::MaybeSome;

#[allow(clippy::module_name_repetitions, clippy::type_complexity)]
#[derive(rust_cuda::common::LendRustToCuda)]
#[cuda(free = "ReportSpeciation", free = "ReportDispersal")]
pub struct EventBuffer<ReportSpeciation: Boolean, ReportDispersal: Boolean> {
    #[cuda(embed)]
    event_mask: SplitSliceOverCudaThreadsDynamicStride<CudaExchangeBuffer<bool, true, true>>,
    #[cuda(embed)]
    event_buffer: SplitSliceOverCudaThreadsDynamicStride<
        CudaExchangeBuffer<
            MaybeSome<<EventBuffer<ReportSpeciation, ReportDispersal> as EventType>::Event>,
            false,
            true,
        >,
    >,
    max_events: usize,
    event_counter: usize,
}

mod sealed {
    struct Assert<const COND: bool>;
    trait AssertTrue {}
    impl AssertTrue for Assert<true> {}

    pub trait AlignedToU64 {}
    impl<T> AlignedToU64 for T
    where
        Assert<{ core::mem::size_of::<T>() % 8 == 0 }>: AssertTrue,
        Assert<{ core::mem::align_of::<T>() == 8 }>: AssertTrue,
    {
    }

    pub trait Array<T> {
        fn len() -> usize;
        fn as_mut_ptr(self: *mut Self) -> *mut T;
    }
    impl<const N: usize, T> Array<T> for [T; N] {
        fn len() -> usize {
            N
        }

        fn as_mut_ptr(self: *mut Self) -> *mut T {
            self.cast()
        }
    }
}

pub trait AlignedToU64: sealed::AlignedToU64 {}
impl<T: sealed::AlignedToU64> AlignedToU64 for T {}

pub trait Array<T>: sealed::Array<T> {
    fn len() -> usize;
    fn as_mut_ptr(self: *mut Self) -> *mut T;
}
impl<T, A: sealed::Array<T>> Array<T> for A {
    fn len() -> usize {
        <A as sealed::Array<T>>::len()
    }

    fn as_mut_ptr(self: *mut Self) -> *mut T {
        <A as sealed::Array<T>>::as_mut_ptr(self)
    }
}

pub trait EventType {
    type Event: 'static
        + ~const rust_cuda::const_type_layout::TypeGraphLayout
        + rust_cuda::safety::StackOnly
        + Into<TypedEvent>
        + Into<PackedEvent>
        + Ord
        + Clone
        + AlignedToU64;

    // const SHARED_LIMIT: usize;

    type SharedBuffer<T: 'static>: 'static + Array<T>;
}

impl<ReportSpeciation: Boolean, ReportDispersal: Boolean> EventType
    for EventBuffer<ReportSpeciation, ReportDispersal>
{
    default type Event = PackedEvent;
    // default const SHARED_LIMIT: usize = 0;

    default type SharedBuffer<T: 'static> = [T; 0];
}

const fn prev_power_of_two(x: usize) -> usize {
    if x.is_power_of_two() {
        x
    } else {
        x.next_power_of_two() / 2
    }
}

impl EventType for EventBuffer<False, False> {
    type Event = PackedEvent;
    type SharedBuffer<T: 'static> =
        [T; prev_power_of_two(48 * 1024 / (core::mem::size_of::<Self::Event>() + 1))];
}

impl EventType for EventBuffer<False, True> {
    type Event = PackedEvent;
    type SharedBuffer<T: 'static> =
        [T; prev_power_of_two(48 * 1024 / (core::mem::size_of::<Self::Event>() + 1))];
}

impl EventType for EventBuffer<True, False> {
    type Event = SpeciationEvent;
    type SharedBuffer<T: 'static> =
        [T; prev_power_of_two(48 * 1024 / (core::mem::size_of::<Self::Event>() + 1))];
}

impl EventType for EventBuffer<True, True> {
    type Event = PackedEvent;
    type SharedBuffer<T: 'static> =
        [T; prev_power_of_two(48 * 1024 / (core::mem::size_of::<Self::Event>() + 1))];
}

impl<ReportSpeciation: Boolean, ReportDispersal: Boolean> fmt::Debug
    for EventBuffer<ReportSpeciation, ReportDispersal>
{
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_struct("EventBuffer")
            .field("max_events", &self.max_events)
            .field("event_counter", &self.event_counter)
            .finish()
    }
}

#[cfg(not(target_os = "cuda"))]
impl<ReportSpeciation: Boolean, ReportDispersal: Boolean>
    EventBuffer<ReportSpeciation, ReportDispersal>
{
    /// # Errors
    /// Returns a `rustacuda::errors::CudaError` iff an error occurs inside CUDA
    pub fn new(
        block_size: &BlockSize,
        grid_size: &GridSize,
        max_events: usize,
    ) -> CudaResult<Self> {
        let block_size = (block_size.x * block_size.y * block_size.z) as usize;
        let grid_size = (grid_size.x * grid_size.y * grid_size.z) as usize;

        #[allow(clippy::bool_to_int_with_if)]
        let max_events = if ReportDispersal::VALUE {
            max_events
        } else if ReportSpeciation::VALUE {
            1_usize
        } else {
            0_usize
        };

        let event_capacity = max_events * block_size * grid_size;

        let mut event_buffer = alloc::vec::Vec::with_capacity(event_capacity);
        event_buffer.resize_with(event_capacity, || MaybeSome::None);

        Ok(Self {
            event_mask: SplitSliceOverCudaThreadsDynamicStride::new(
                CudaExchangeBuffer::new(&false, event_capacity)?,
                max_events,
            ),
            event_buffer: SplitSliceOverCudaThreadsDynamicStride::new(
                CudaExchangeBuffer::from_vec(event_buffer)?,
                max_events,
            ),
            max_events,
            event_counter: 0_usize,
        })
    }

    pub fn report_events_unordered<P>(&mut self, reporter: &mut P)
    where
        P: Reporter<ReportSpeciation = ReportSpeciation, ReportDispersal = ReportDispersal>,
    {
        for (mask, event) in self.event_mask.iter_mut().zip(self.event_buffer.iter()) {
            if *mask.read() {
                let event: TypedEvent = unsafe { event.read().assume_some_read() }.into();

                match event {
                    TypedEvent::Speciation(ref speciation) => {
                        reporter.report_speciation(speciation.into());
                    },
                    TypedEvent::Dispersal(ref dispersal) => {
                        reporter.report_dispersal(dispersal.into());
                    },
                }
            }

            mask.write(false);
        }
    }

    pub fn max_events_per_individual(&self) -> usize {
        self.max_events
    }
}

#[cfg(not(target_os = "cuda"))]
impl<ReportSpeciation: Boolean, ReportDispersal: Boolean>
    EventBuffer<ReportSpeciation, ReportDispersal>
{
    pub fn sort_events(&mut self) {
        let mut events: alloc::vec::Vec<<Self as EventType>::Event> = alloc::vec::Vec::new();

        for (mask, event) in self.event_mask.iter_mut().zip(self.event_buffer.iter()) {
            if *mask.read() {
                events.push(unsafe { event.read().assume_some_read() });
            }

            mask.write(false);
        }

        events.sort_unstable();

        for ((event, mask), scratch) in events
            .into_iter()
            .zip(self.event_mask.iter_mut())
            .zip(self.event_buffer.iter_mut())
        {
            *scratch.as_scratch_mut() = MaybeSome::Some(event);
            mask.write(true);
        }
    }
}

#[cfg(target_os = "cuda")]
impl<ReportSpeciation: Boolean, ReportDispersal: Boolean>
    EventBuffer<ReportSpeciation, ReportDispersal>
{
    /// Bitonic sort combined pre-sort for shared memory, based on
    /// <https://github.com/NVIDIA/cuda-samples/blob/81992093d2b8c33cab22dbf6852c070c330f1715/Samples/2_Concepts_and_Techniques/sortingNetworks/bitonicSort.cu#L93-L152>
    ///
    /// # Safety
    ///
    /// Only one call per kernel launch is safe without further synchronisation.
    ///
    /// # Panics
    ///
    /// Traps if the thread block size does not equal
    /// `<Self as EventType>::SharedBuffer::<()>::len() / 2`.
    #[allow(clippy::too_many_lines)]
    pub unsafe fn bitonic_sort_events_shared_prep(&mut self) {
        use core::cmp::Ordering;

        let shared_buffer_len = <Self as EventType>::SharedBuffer::<()>::len();

        let thread = rust_cuda::device::thread::Thread::this();
        let thread_idx = thread.idx();
        let thread_block = thread.block();
        let block_dim = thread_block.dim();
        let block_idx = thread_block.idx();
        let block_grid = thread_block.grid();
        let grid_dim = block_grid.dim();

        if shared_buffer_len != (block_dim.size() * 2) {
            core::arch::nvptx::trap();
        }

        let block_idx = block_idx.as_id(&grid_dim);
        let thread_idx = thread_idx.as_id(&block_dim);

        let idx = block_idx * shared_buffer_len + thread_idx;

        let shared_mask: rust_cuda::utils::shared::r#static::ThreadBlockShared<
            <Self as EventType>::SharedBuffer<bool>,
        > = rust_cuda::utils::shared::r#static::ThreadBlockShared::new_uninit();
        let shared_mask_array: *mut bool = shared_mask.as_mut_ptr().cast();
        let shared_buffer: rust_cuda::utils::shared::r#static::ThreadBlockShared<
            <Self as EventType>::SharedBuffer<MaybeSome<<Self as EventType>::Event>>,
        > = rust_cuda::utils::shared::r#static::ThreadBlockShared::new_uninit();
        let shared_buffer_array: *mut MaybeSome<<Self as EventType>::Event> =
            shared_buffer.as_mut_ptr().cast();

        *shared_mask_array.add(thread_idx) = match self.event_mask.alias_unchecked().get(idx) {
            None => false,
            Some(mask) => *mask.read(),
        };
        if let Some(event) = self.event_buffer.alias_unchecked().get(idx) {
            let ptr_src = event.as_uninit().as_ptr();
            let ptr_dst = shared_buffer_array.add(thread_idx);

            core::ptr::copy_nonoverlapping(ptr_src, ptr_dst, 1);
        };
        *shared_mask_array.add(thread_idx + (shared_buffer_len / 2)) = match self
            .event_mask
            .alias_unchecked()
            .get(idx + (shared_buffer_len / 2))
        {
            None => false,
            Some(mask) => *mask.read(),
        };
        if let Some(event) = self
            .event_buffer
            .alias_unchecked()
            .get(idx + (shared_buffer_len / 2))
        {
            let ptr_src = event.as_uninit().as_ptr();
            let ptr_dst = shared_buffer_array.add(thread_idx + (shared_buffer_len / 2));

            core::ptr::copy_nonoverlapping(ptr_src, ptr_dst, 1);
        };

        let mut size = 2;

        while size < shared_buffer_len {
            let dir = if (thread_idx & (size / 2)) == 0 {
                Ordering::Less
            } else {
                Ordering::Greater
            };

            let mut stride = size >> 1;

            while stride > 0 {
                ::core::arch::nvptx::_syncthreads();

                let pos_a = 2 * thread_idx - (thread_idx & (stride - 1));
                let pos_b = pos_a + stride;

                let mask_a: bool = *shared_mask_array.add(pos_a);
                let mask_b: bool = *shared_mask_array.add(pos_b);

                let cmp = match (mask_a, mask_b) {
                    (false, false) => Ordering::Equal,
                    (false, true) => Ordering::Greater,
                    (true, false) => Ordering::Less,
                    (true, true) => {
                        // Safety: both masks indicate that the two events exist
                        let event_a: &<Self as EventType>::Event =
                            unsafe { (*shared_buffer_array.add(pos_a)).assume_some_ref() };
                        let event_b: &<Self as EventType>::Event =
                            unsafe { (*shared_buffer_array.add(pos_b)).assume_some_ref() };

                        event_a.cmp(event_b)
                    },
                };

                if cmp == dir {
                    *shared_mask_array.add(pos_a) = mask_b;
                    *shared_mask_array.add(pos_b) = mask_a;

                    let ptr_a: *mut u64 = shared_buffer_array.add(pos_a).cast();
                    let ptr_b: *mut u64 = shared_buffer_array.add(pos_b).cast();

                    // Manual swap implementation that can be unrolled without local memory
                    // Safety: AlignedToU64 guarantees that both events are aligned to u64
                    //         and can be copied as multiples of u64
                    for i in 0..(core::mem::size_of::<<Self as EventType>::Event>() / 8) {
                        let swap = *ptr_a.add(i);
                        *ptr_a.add(i) = *ptr_b.add(i);
                        *ptr_b.add(i) = swap;
                    }
                }

                stride >>= 1;
            }

            size <<= 1;
        }

        let dir = if (block_idx & 1) == 0 {
            Ordering::Less
        } else {
            Ordering::Greater
        };

        let mut stride = shared_buffer_len >> 1;

        while stride > 0 {
            ::core::arch::nvptx::_syncthreads();

            let pos_a = 2 * thread_idx - (thread_idx & (stride - 1));
            let pos_b = pos_a + stride;

            let mask_a: bool = *shared_mask_array.add(pos_a);
            let mask_b: bool = *shared_mask_array.add(pos_b);

            let cmp = match (mask_a, mask_b) {
                (false, false) => Ordering::Equal,
                (false, true) => Ordering::Greater,
                (true, false) => Ordering::Less,
                (true, true) => {
                    // Safety: both masks indicate that the two events exist
                    let event_a: &<Self as EventType>::Event =
                        unsafe { (*shared_buffer_array.add(pos_a)).assume_some_ref() };
                    let event_b: &<Self as EventType>::Event =
                        unsafe { (*shared_buffer_array.add(pos_b)).assume_some_ref() };

                    event_a.cmp(event_b)
                },
            };

            if cmp == dir {
                *shared_mask_array.add(pos_a) = mask_b;
                *shared_mask_array.add(pos_b) = mask_a;

                let ptr_a: *mut u64 = shared_buffer_array.add(pos_a).cast();
                let ptr_b: *mut u64 = shared_buffer_array.add(pos_b).cast();

                // Manual swap implementation that can be unrolled without local memory
                // Safety: AlignedToU64 guarantees that both events are aligned to u64
                //         and can be copied as multiples of u64
                for i in 0..(core::mem::size_of::<<Self as EventType>::Event>() / 8) {
                    let swap = *ptr_a.add(i);
                    *ptr_a.add(i) = *ptr_b.add(i);
                    *ptr_b.add(i) = swap;
                }
            }

            stride >>= 1;
        }

        ::core::arch::nvptx::_syncthreads();

        if let Some(mask) = self.event_mask.alias_mut_unchecked().get_mut(idx) {
            mask.write(*shared_mask_array.add(thread_idx));
        }
        if let Some(event) = self.event_buffer.alias_mut_unchecked().get_mut(idx) {
            event.write(core::ptr::read(shared_buffer_array.add(thread_idx)));
        }
        if let Some(mask) = self
            .event_mask
            .alias_mut_unchecked()
            .get_mut(idx + (shared_buffer_len / 2))
        {
            mask.write(*shared_mask_array.add(thread_idx + (shared_buffer_len / 2)));
        }
        if let Some(event) = self
            .event_buffer
            .alias_mut_unchecked()
            .get_mut(idx + (shared_buffer_len / 2))
        {
            event.write(core::ptr::read(
                shared_buffer_array.add(thread_idx + (shared_buffer_len / 2)),
            ));
        }
    }

    /// Bitonic sort combined merge step for shared memory, based on
    /// <https://github.com/NVIDIA/cuda-samples/blob/81992093d2b8c33cab22dbf6852c070c330f1715/Samples/2_Concepts_and_Techniques/sortingNetworks/bitonicSort.cu#L179-L220>
    ///
    /// # Safety
    ///
    /// All CUDA threads must call this method with the same size argument.
    /// Only one call per kernel launch is safe without further synchronisation.
    ///
    /// # Panics
    ///
    /// Traps if the thread block size does not equal
    /// `<Self as EventType>::SharedBuffer::<()>::len() / 2`.
    #[allow(clippy::too_many_lines)]
    pub unsafe fn bitonic_sort_events_shared_step(&mut self, size: usize) {
        use core::cmp::Ordering;

        let shared_buffer_len = <Self as EventType>::SharedBuffer::<()>::len();

        let thread = rust_cuda::device::thread::Thread::this();
        let thread_idx = thread.idx();
        let thread_block = thread.block();
        let block_dim = thread_block.dim();
        let block_idx = thread_block.idx();
        let block_grid = thread_block.grid();
        let grid_dim = block_grid.dim();

        if shared_buffer_len != (block_dim.size() * 2) {
            core::arch::nvptx::trap();
        }

        let block_idx = block_idx.as_id(&grid_dim);
        let thread_idx = thread_idx.as_id(&block_dim);

        let idx = block_idx * shared_buffer_len + thread_idx;

        let shared_mask: rust_cuda::utils::shared::r#static::ThreadBlockShared<
            <Self as EventType>::SharedBuffer<bool>,
        > = rust_cuda::utils::shared::r#static::ThreadBlockShared::new_uninit();
        let shared_mask_array: *mut bool = shared_mask.as_mut_ptr().cast();
        let shared_buffer: rust_cuda::utils::shared::r#static::ThreadBlockShared<
            <Self as EventType>::SharedBuffer<MaybeSome<<Self as EventType>::Event>>,
        > = rust_cuda::utils::shared::r#static::ThreadBlockShared::new_uninit();
        let shared_buffer_array: *mut MaybeSome<<Self as EventType>::Event> =
            shared_buffer.as_mut_ptr().cast();

        *shared_mask_array.add(thread_idx) = match self.event_mask.alias_unchecked().get(idx) {
            None => false,
            Some(mask) => *mask.read(),
        };
        if let Some(event) = self.event_buffer.alias_unchecked().get(idx) {
            let ptr_src = event.as_uninit().as_ptr();
            let ptr_dst = shared_buffer_array.add(thread_idx);

            core::ptr::copy_nonoverlapping(ptr_src, ptr_dst, 1);
        };
        *shared_mask_array.add(thread_idx + (shared_buffer_len / 2)) = match self
            .event_mask
            .alias_unchecked()
            .get(idx + (shared_buffer_len / 2))
        {
            None => false,
            Some(mask) => *mask.read(),
        };
        if let Some(event) = self
            .event_buffer
            .alias_unchecked()
            .get(idx + (shared_buffer_len / 2))
        {
            let ptr_src = event.as_uninit().as_ptr();
            let ptr_dst = shared_buffer_array.add(thread_idx + (shared_buffer_len / 2));

            core::ptr::copy_nonoverlapping(ptr_src, ptr_dst, 1);
        };

        let pos = (block_idx * block_dim.size() + thread_idx)
            & ((self.event_mask.alias_unchecked().len().next_power_of_two() / 2) - 1);
        let dir = if (pos & (size / 2)) == 0 {
            Ordering::Greater
        } else {
            Ordering::Less
        };

        let mut stride = shared_buffer_len >> 1;

        while stride > 0 {
            ::core::arch::nvptx::_syncthreads();

            let pos_a = 2 * thread_idx - (thread_idx & (stride - 1));
            let pos_b = pos_a + stride;

            let mask_a: bool = *shared_mask_array.add(pos_a);
            let mask_b: bool = *shared_mask_array.add(pos_b);

            let cmp = match (mask_a, mask_b) {
                (false, false) => Ordering::Equal,
                (false, true) => Ordering::Greater,
                (true, false) => Ordering::Less,
                (true, true) => {
                    // Safety: both masks indicate that the two events exist
                    let event_a: &<Self as EventType>::Event =
                        unsafe { (*shared_buffer_array.add(pos_a)).assume_some_ref() };
                    let event_b: &<Self as EventType>::Event =
                        unsafe { (*shared_buffer_array.add(pos_b)).assume_some_ref() };

                    event_a.cmp(event_b)
                },
            };

            if cmp == dir {
                *shared_mask_array.add(pos_a) = mask_b;
                *shared_mask_array.add(pos_b) = mask_a;

                let ptr_a: *mut u64 = shared_buffer_array.add(pos_a).cast();
                let ptr_b: *mut u64 = shared_buffer_array.add(pos_b).cast();

                // Manual swap implementation that can be unrolled without local memory
                // Safety: AlignedToU64 guarantees that both events are aligned to u64
                //         and can be copied as multiples of u64
                for i in 0..(core::mem::size_of::<<Self as EventType>::Event>() / 8) {
                    let swap = *ptr_a.add(i);
                    *ptr_a.add(i) = *ptr_b.add(i);
                    *ptr_b.add(i) = swap;
                }
            }

            stride >>= 1;
        }

        ::core::arch::nvptx::_syncthreads();

        if let Some(mask) = self.event_mask.alias_mut_unchecked().get_mut(idx) {
            mask.write(*shared_mask_array.add(thread_idx));
        }
        if let Some(event) = self.event_buffer.alias_mut_unchecked().get_mut(idx) {
            event.write(core::ptr::read(shared_buffer_array.add(thread_idx)));
        }
        if let Some(mask) = self
            .event_mask
            .alias_mut_unchecked()
            .get_mut(idx + (shared_buffer_len / 2))
        {
            mask.write(*shared_mask_array.add(thread_idx + (shared_buffer_len / 2)));
        }
        if let Some(event) = self
            .event_buffer
            .alias_mut_unchecked()
            .get_mut(idx + (shared_buffer_len / 2))
        {
            event.write(core::ptr::read(
                shared_buffer_array.add(thread_idx + (shared_buffer_len / 2)),
            ));
        }
    }

    /// Bitonic sort single merge step for global memory, based on
    /// <https://github.com/NVIDIA/cuda-samples/blob/81992093d2b8c33cab22dbf6852c070c330f1715/Samples/2_Concepts_and_Techniques/sortingNetworks/bitonicSort.cu#L154-L177>
    ///
    /// # Safety
    ///
    /// All CUDA threads must call this method with the same size and stride
    /// arguments. Only one call per kernel launch is safe without further
    /// synchronisation.
    pub unsafe fn bitonic_sort_events_step(&mut self, size: usize, stride: usize) {
        use core::cmp::Ordering;

        let idx = rust_cuda::device::thread::Thread::this().index();

        let pos = idx & ((self.event_mask.alias_unchecked().len().next_power_of_two() / 2) - 1);

        let dir = if (pos & (size / 2)) == 0 {
            Ordering::Greater
        } else {
            Ordering::Less
        };

        let pos_a = 2 * idx - (idx & (stride - 1));
        let pos_b = pos_a + stride;

        if (pos_a < self.event_mask.alias_unchecked().len())
            && (pos_b < self.event_mask.alias_unchecked().len())
        {
            let mask_a: bool = *self
                .event_mask
                .alias_unchecked()
                .get_unchecked(pos_a)
                .read();
            let mask_b: bool = *self
                .event_mask
                .alias_unchecked()
                .get_unchecked(pos_b)
                .read();

            let cmp = match (mask_a, mask_b) {
                (false, false) => Ordering::Equal,
                (false, true) => Ordering::Greater,
                (true, false) => Ordering::Less,
                (true, true) => {
                    // Safety: both masks indicate that the two events exist
                    let event_a: &<Self as EventType>::Event = unsafe {
                        self.event_buffer
                            .alias_unchecked()
                            .get_unchecked(pos_a)
                            .as_uninit()
                            .assume_init_ref()
                            .assume_some_ref()
                    };
                    let event_b: &<Self as EventType>::Event = unsafe {
                        self.event_buffer
                            .alias_unchecked()
                            .get_unchecked(pos_b)
                            .as_uninit()
                            .assume_init_ref()
                            .assume_some_ref()
                    };

                    event_a.cmp(event_b)
                },
            };

            if cmp == dir {
                self.event_mask
                    .alias_mut_unchecked()
                    .get_unchecked_mut(pos_a)
                    .write(mask_b);
                self.event_mask
                    .alias_mut_unchecked()
                    .get_unchecked_mut(pos_b)
                    .write(mask_a);

                let ptr_a: *mut u64 = self
                    .event_buffer
                    .alias_mut_unchecked()
                    .as_mut_ptr()
                    .add(pos_a)
                    .cast();
                let ptr_b: *mut u64 = self
                    .event_buffer
                    .alias_mut_unchecked()
                    .as_mut_ptr()
                    .add(pos_b)
                    .cast();

                // Manual swap implementation that can be unrolled without local memory
                // Safety: AlignedToU64 guarantees that both events are aligned to u64
                //         and can be copied as multiples of u64
                for i in 0..(core::mem::size_of::<<Self as EventType>::Event>() / 8) {
                    let swap = *ptr_a.add(i);
                    *ptr_a.add(i) = *ptr_b.add(i);
                    *ptr_b.add(i) = swap;
                }
            }
        }
    }

    #[allow(clippy::too_many_lines)]
    /// Odd-Even sort single merge step for global memory, based on
    /// <https://github.com/NVIDIA/cuda-samples/blob/81992093d2b8c33cab22dbf6852c070c330f1715/Samples/2_Concepts_and_Techniques/sortingNetworks/oddEvenMergeSort.cu#L95-L137>
    ///
    /// # Safety
    ///
    /// All CUDA threads must call this method with the same size and stride
    /// arguments. Only one call per kernel launch is safe without further
    /// synchronisation.
    pub unsafe fn odd_even_sort_events_step(&mut self, size: usize, stride: usize) {
        use core::cmp::Ordering;

        let idx = rust_cuda::device::thread::Thread::this().index();

        let pos = 2 * idx - (idx & (stride - 1));

        let (pos_a, pos_b) = if stride < (size / 2) {
            (pos.wrapping_sub(stride), pos)
        } else {
            (pos, pos + stride)
        };

        let offset = idx & ((size / 2) - 1);

        if (pos_a < self.event_mask.alias_unchecked().len())
            && (pos_b < self.event_mask.alias_unchecked().len())
            && ((stride >= (size / 2)) || (offset >= stride))
        {
            let mask_a: bool = *self
                .event_mask
                .alias_unchecked()
                .get_unchecked(pos_a)
                .read();
            let mask_b: bool = *self
                .event_mask
                .alias_unchecked()
                .get_unchecked(pos_b)
                .read();

            let cmp = match (mask_a, mask_b) {
                (false, false) => Ordering::Equal,
                (false, true) => Ordering::Greater,
                (true, false) => Ordering::Less,
                (true, true) => {
                    // Safety: both masks indicate that the two events exist
                    let event_a: &<Self as EventType>::Event = unsafe {
                        self.event_buffer
                            .alias_unchecked()
                            .get_unchecked(pos_a)
                            .as_uninit()
                            .assume_init_ref()
                            .assume_some_ref()
                    };
                    let event_b: &<Self as EventType>::Event = unsafe {
                        self.event_buffer
                            .alias_unchecked()
                            .get_unchecked(pos_b)
                            .as_uninit()
                            .assume_init_ref()
                            .assume_some_ref()
                    };

                    event_a.cmp(event_b)
                },
            };

            if let Ordering::Greater = cmp {
                self.event_mask
                    .alias_mut_unchecked()
                    .get_unchecked_mut(pos_a)
                    .write(mask_b);
                self.event_mask
                    .alias_mut_unchecked()
                    .get_unchecked_mut(pos_b)
                    .write(mask_a);

                let ptr_a: *mut u64 = self
                    .event_buffer
                    .alias_mut_unchecked()
                    .as_mut_ptr()
                    .add(pos_a)
                    .cast();
                let ptr_b: *mut u64 = self
                    .event_buffer
                    .alias_mut_unchecked()
                    .as_mut_ptr()
                    .add(pos_b)
                    .cast();

                // Manual swap implementation that can be unrolled without local memory
                // Safety: AlignedToU64 guarantees that both events are aligned to u64
                //         and can be copied as multiples of u64
                for i in 0..(core::mem::size_of::<<Self as EventType>::Event>() / 8) {
                    let swap = *ptr_a.add(i);
                    *ptr_a.add(i) = *ptr_b.add(i);
                    *ptr_b.add(i) = swap;
                }
            }
        }
    }
}

#[cfg(target_os = "cuda")]
impl<ReportSpeciation: Boolean, ReportDispersal: Boolean> Reporter
    for EventBuffer<ReportSpeciation, ReportDispersal>
{
    impl_report!([default] speciation(&mut self, _event: Ignored) {});

    impl_report!([default] dispersal(&mut self, _event: Ignored) {});

    impl_report!([default] progress(&mut self, _progress: Ignored) {});
}

#[cfg(target_os = "cuda")]
impl Reporter for EventBuffer<False, True> {
    impl_report!(
        #[debug_requires(
            self.event_counter < self.max_events,
            "does not report extraneous dispersal events"
        )]
        dispersal(&mut self, event: Used) {
            if let Some(mask) = self.event_mask.get_mut(self.event_counter) {
                mask.write(true);

                unsafe {
                    self.event_buffer.get_unchecked_mut(self.event_counter)
                }.write(MaybeSome::Some(event.clone().into()));
            }

            self.event_counter += 1;
        }
    );
}

#[cfg(target_os = "cuda")]
impl Reporter for EventBuffer<True, False> {
    impl_report!(
        #[debug_requires(
            self.event_counter == 0,
            "does not report extraneous speciation events"
        )]
        speciation(&mut self, event: Used) {
            if let Some(mask) = self.event_mask.get_mut(0) {
                mask.write(true);

                unsafe {
                    self.event_buffer.get_unchecked_mut(0)
                }.write(MaybeSome::Some(event.clone()));
            }

            self.event_counter = self.max_events;
        }
    );
}

#[cfg(target_os = "cuda")]
impl Reporter for EventBuffer<True, True> {
    impl_report!(
        #[debug_requires(
            self.event_counter < self.max_events,
            "does not report extraneous speciation events"
        )]
        speciation(&mut self, event: Used) {
            if let Some(mask) = self.event_mask.get_mut(self.event_counter) {
                mask.write(true);

                unsafe {
                    self.event_buffer.get_unchecked_mut(self.event_counter)
                }.write(MaybeSome::Some(event.clone().into()));
            }

            self.event_counter = self.max_events;
        }
    );

    impl_report!(
        #[debug_requires(
            self.event_counter < self.max_events,
            "does not report extraneous dispersal events"
        )]
        dispersal(&mut self, event: Used) {
            if let Some(mask) = self.event_mask.get_mut(self.event_counter) {
                mask.write(true);

                unsafe {
                    self.event_buffer.get_unchecked_mut(self.event_counter)
                }.write(MaybeSome::Some(event.clone().into()));
            }

            self.event_counter += 1;
        }
    );
}
