use alloc::{collections::VecDeque, vec::Vec};
use core::{fmt, marker::PhantomData, ops::ControlFlow};
use necsim_core_bond::NonNegativeF64;

use necsim_core::{
    event::{PackedEvent, TypedEvent},
    impl_report,
    reporter::Reporter,
};

use necsim_partitioning_core::LocalPartition;

use super::WaterLevelReporterProxy;

#[derive(Clone, Copy)]
struct Run {
    start: usize,
    len: usize,
}

#[allow(clippy::module_name_repetitions)]
pub struct LiveWaterLevelReporterProxy<'l, 'p, R: Reporter, P: LocalPartition<'p, R>> {
    water_level: NonNegativeF64,
    slow_events: Vec<PackedEvent>,
    tmp_events: Vec<PackedEvent>,
    run: Run,
    runs: Vec<Run>,
    overflow: VecDeque<Run>,
    sort_batch_size: usize,
    fast_events: Vec<PackedEvent>,

    local_partition: &'l mut P,
    _marker: PhantomData<(&'p (), R)>,
}

impl<'l, 'p, R: Reporter, P: LocalPartition<'p, R>> fmt::Debug
    for LiveWaterLevelReporterProxy<'l, 'p, R, P>
{
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        struct EventBufferLen(usize);

        impl fmt::Debug for EventBufferLen {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                write!(f, "Vec<PackedEvent; {}>", self.0)
            }
        }

        fmt.debug_struct(stringify!(LiveWaterLevelReporterProxy))
            .field("water_level", &self.water_level)
            .field("runs", &self.runs.len())
            .field("overflow", &self.overflow.len())
            .field("slow_events", &EventBufferLen(self.slow_events.len()))
            .field("fast_events", &EventBufferLen(self.fast_events.len()))
            .finish()
    }
}

impl<'l, 'p, R: Reporter, P: LocalPartition<'p, R>> Reporter
    for LiveWaterLevelReporterProxy<'l, 'p, R, P>
{
    impl_report!(speciation(&mut self, speciation: MaybeUsed<R::ReportSpeciation>) {
        let speciation: PackedEvent = speciation.clone().into();

        if speciation.event_time() < self.water_level {
            let new_run = self.run.len > self.sort_batch_size; // self.slow_events.last().map_or(true, |prev| prev > &speciation);

            if new_run {
                let old_run = core::mem::replace(&mut self.run, Run {
                    start: self.slow_events.len(),
                    len: 1,
                });
                self.overflow.push_back(old_run);
            } else {
                self.run.len += 1;
            }

            self.slow_events.push(speciation);
        } else {
            self.fast_events.push(speciation);
        }
    });

    impl_report!(dispersal(&mut self, dispersal: MaybeUsed<R::ReportDispersal>) {
        let dispersal: PackedEvent = dispersal.clone().into();

        if dispersal.event_time() < self.water_level {
            let new_run = self.run.len > self.sort_batch_size; // self.slow_events.last().map_or(true, |prev| prev > &dispersal);

            if new_run {
                let old_run = core::mem::replace(&mut self.run, Run {
                    start: self.slow_events.len(),
                    len: 1,
                });
                self.overflow.push_back(old_run);
            } else {
                self.run.len += 1;
            }

            self.slow_events.push(dispersal);
        } else {
            self.fast_events.push(dispersal);
        }
    });

    impl_report!(progress(&mut self, _progress: Ignored) {});
}

impl<'l, 'p, R: Reporter, P: LocalPartition<'p, R>> LiveWaterLevelReporterProxy<'l, 'p, R, P> {
    #[inline]
    fn collapse(&self, force_merge: bool) -> Option<usize> {
        let n = self.runs.len();
        if n >= 2
            && (force_merge
                || self.runs[n - 2].len <= self.runs[n - 1].len
                || (n >= 3 && self.runs[n - 3].len <= self.runs[n - 2].len + self.runs[n - 1].len)
                || (n >= 4 && self.runs[n - 4].len <= self.runs[n - 3].len + self.runs[n - 2].len))
        {
            if n >= 3 && self.runs[n - 3].len < self.runs[n - 1].len {
                Some(n - 3)
            } else {
                Some(n - 2)
            }
        } else {
            None
        }
    }

    unsafe fn merge<T, F>(v: &mut [T], mid: usize, buf: *mut T, is_less: &mut F)
    where
        F: FnMut(&T, &T) -> bool,
    {
        unsafe fn get_and_increment<T>(ptr: &mut *mut T) -> *mut T {
            let old = *ptr;
            *ptr = unsafe { ptr.add(1) };
            old
        }

        unsafe fn decrement_and_get<T>(ptr: &mut *mut T) -> *mut T {
            *ptr = unsafe { ptr.sub(1) };
            *ptr
        }

        // When dropped, copies the range `start..end` into `dest..`.
        struct MergeHole<T> {
            start: *mut T,
            end: *mut T,
            dest: *mut T,
        }

        impl<T> Drop for MergeHole<T> {
            fn drop(&mut self) {
                // `T` is not a zero-sized type, and these are pointers into a slice's elements.
                unsafe {
                    let len = self.end.sub_ptr(self.start);
                    core::ptr::copy_nonoverlapping(self.start, self.dest, len);
                }
            }
        }

        let len = v.len();
        let v = v.as_mut_ptr();
        let (v_mid, v_end) = unsafe { (v.add(mid), v.add(len)) };

        // The merge process first copies the shorter run into `buf`. Then it traces the
        // newly copied run and the longer run forwards (or backwards),
        // comparing their next unconsumed elements and copying the lesser (or
        // greater) one into `v`.
        //
        // As soon as the shorter run is fully consumed, the process is done. If the
        // longer run gets consumed first, then we must copy whatever is left of
        // the shorter run into the remaining hole in `v`.
        //
        // Intermediate state of the process is always tracked by `hole`, which serves
        // two purposes: 1. Protects integrity of `v` from panics in `is_less`.
        // 2. Fills the remaining hole in `v` if the longer run gets consumed first.
        //
        // Panic safety:
        //
        // If `is_less` panics at any point during the process, `hole` will get dropped
        // and fill the hole in `v` with the unconsumed range in `buf`, thus
        // ensuring that `v` still holds every object it initially held exactly
        // once.
        let mut hole;

        if mid <= len - mid {
            // The left run is shorter.
            unsafe {
                core::ptr::copy_nonoverlapping(v, buf, mid);
                hole = MergeHole {
                    start: buf,
                    end: buf.add(mid),
                    dest: v,
                };
            }

            // Initially, these pointers point to the beginnings of their arrays.
            let left = &mut hole.start;
            let mut right = v_mid;
            let out = &mut hole.dest;

            while *left < hole.end && right < v_end {
                // Consume the lesser side.
                // If equal, prefer the left run to maintain stability.
                unsafe {
                    let to_copy = if is_less(&*right, &**left) {
                        get_and_increment(&mut right)
                    } else {
                        get_and_increment(left)
                    };
                    core::ptr::copy_nonoverlapping(to_copy, get_and_increment(out), 1);
                }
            }
        } else {
            // The right run is shorter.
            unsafe {
                core::ptr::copy_nonoverlapping(v_mid, buf, len - mid);
                hole = MergeHole {
                    start: buf,
                    end: buf.add(len - mid),
                    dest: v_mid,
                };
            }

            // Initially, these pointers point past the ends of their arrays.
            let left = &mut hole.dest;
            let right = &mut hole.end;
            let mut out = v_end;

            while v < *left && buf < *right {
                // Consume the greater side.
                // If equal, prefer the right run to maintain stability.
                unsafe {
                    let to_copy = if is_less(&*right.sub(1), &*left.sub(1)) {
                        decrement_and_get(left)
                    } else {
                        decrement_and_get(right)
                    };
                    core::ptr::copy_nonoverlapping(to_copy, decrement_and_get(&mut out), 1);
                }
            }
        }
        // Finally, `hole` gets dropped. If the shorter run was not fully
        // consumed, whatever remains of it will now be copied into the
        // hole in `v`.
    }

    fn sort_slow_events_step(&mut self, force_merge: bool) -> ControlFlow<()> {
        let Some(r) = self.collapse(force_merge && self.overflow.is_empty() && self.run.len == 0) else {
            let next_run = match self.overflow.pop_front() {
                Some(next_run) => next_run,
                None if self.run.len > 0 => core::mem::replace(&mut self.run, Run { start: self.slow_events.len(), len: 0 }),
                None => return ControlFlow::Break(()),
            };

            self.slow_events[next_run.start..next_run.start+next_run.len].sort_unstable();

            self.runs.push(next_run);

            return ControlFlow::Continue(());
        };

        let left = self.runs[r];
        let right = self.runs[r + 1];

        let min_len = left.len.min(right.len);

        if min_len > self.tmp_events.capacity() {
            self.tmp_events
                .reserve(min_len - self.tmp_events.capacity());
        }

        unsafe {
            Self::merge(
                &mut self.slow_events[left.start..right.start + right.len],
                left.len,
                self.tmp_events.as_mut_ptr(),
                &mut core::cmp::PartialOrd::lt,
            );
        }

        self.runs[r] = Run {
            start: left.start,
            len: left.len + right.len,
        };
        self.runs.remove(r + 1);

        ControlFlow::Continue(())
    }
}

#[contract_trait]
impl<'l, 'p, R: Reporter, P: LocalPartition<'p, R>> WaterLevelReporterProxy<'l, 'p, R, P>
    for LiveWaterLevelReporterProxy<'l, 'p, R, P>
{
    fn new(capacity: usize, local_partition: &'l mut P, sort_batch_size: usize) -> Self {
        info!("Events will be reported using the live water-level algorithm ...");

        Self {
            water_level: NonNegativeF64::zero(),
            slow_events: Vec::with_capacity(capacity),
            tmp_events: Vec::with_capacity(capacity),
            run: Run { start: 0, len: 0 },
            runs: Vec::new(),
            overflow: VecDeque::new(),
            fast_events: Vec::with_capacity(capacity),
            sort_batch_size,

            local_partition,
            _marker: PhantomData::<(&'p (), R)>,
        }
    }

    fn water_level(&self) -> NonNegativeF64 {
        self.water_level
    }

    fn partial_sort_step(&mut self) -> ControlFlow<()> {
        self.sort_slow_events_step(false)
    }

    fn advance_water_level(&mut self, water_level: NonNegativeF64) {
        let mut i = 0;

        // Report all events below the water level in sorted order
        // TODO: Should we detect if no partial sort steps were taken
        //       and revert to a full unstable sort in that case?
        while let ControlFlow::Continue(()) = self.sort_slow_events_step(true) {
            if (i % 100) == 0 {
                info!("{:?}", self);
            }
            i += 1;
        }

        debug_assert!(self.slow_events.is_sorted());

        for event in self.slow_events.drain(..) {
            match event.into() {
                TypedEvent::Speciation(event) => {
                    self.local_partition
                        .get_reporter()
                        .report_speciation(&event.into());
                },
                TypedEvent::Dispersal(event) => {
                    self.local_partition
                        .get_reporter()
                        .report_dispersal(&event.into());
                },
            }
        }

        self.runs.clear();
        self.run = Run { start: 0, len: 0 };

        // Update the water level
        self.water_level = water_level;

        // Move fast events below the new water level into slow events
        for event in self
            .fast_events
            .drain_filter(|event| event.event_time() < water_level)
        {
            let new_run = self.run.len > self.sort_batch_size; // self.slow_events.last().map_or(true, |prev| prev > &event);

            if new_run {
                let old_run = core::mem::replace(
                    &mut self.run,
                    Run {
                        start: self.slow_events.len(),
                        len: 1,
                    },
                );
                self.overflow.push_back(old_run);
            } else {
                self.run.len += 1;
            }

            self.slow_events.push(event);
        }
    }

    fn local_partition(&mut self) -> &mut P {
        self.local_partition
    }
}

impl<'l, 'p, R: Reporter, P: LocalPartition<'p, R>> Drop
    for LiveWaterLevelReporterProxy<'l, 'p, R, P>
{
    fn drop(&mut self) {
        // Report all events below the water level in sorted order
        while let ControlFlow::Continue(()) = self.sort_slow_events_step(true) {}

        debug_assert!(self.slow_events.is_sorted());

        for event in self.slow_events.drain(..) {
            match event.into() {
                TypedEvent::Speciation(event) => {
                    self.local_partition
                        .get_reporter()
                        .report_speciation(&event.into());
                },
                TypedEvent::Dispersal(event) => {
                    self.local_partition
                        .get_reporter()
                        .report_dispersal(&event.into());
                },
            }
        }

        self.runs.clear();
        self.run = Run { start: 0, len: 0 };

        // Report all events above the water level in sorted order
        self.fast_events.sort();

        for event in self.fast_events.drain(..) {
            match event.into() {
                TypedEvent::Speciation(event) => {
                    self.local_partition
                        .get_reporter()
                        .report_speciation(&event.into());
                },
                TypedEvent::Dispersal(event) => {
                    self.local_partition
                        .get_reporter()
                        .report_dispersal(&event.into());
                },
            }
        }
    }
}
