use std::{
    cmp::{Ord, Ordering},
    collections::{BinaryHeap, VecDeque},
    fs::{File, OpenOptions},
    io::BufReader,
    path::{Path, PathBuf},
};

use necsim_core::event::Event;

use anyhow::Result;

#[allow(clippy::module_name_repetitions)]
pub struct EventReplayIterator {
    frontier: BinaryHeap<SortedSegment>,
}

impl EventReplayIterator {
    /// # Errors
    ///
    /// Returns `Err` iff any of the paths could not be read.
    pub fn try_new(paths: &[PathBuf], capacity: usize) -> Result<Self> {
        let mut frontier = BinaryHeap::with_capacity(paths.len());

        for path in paths {
            frontier.push(SortedSegment::new(path, capacity)?);
        }

        Ok(Self { frontier })
    }
}

impl Iterator for EventReplayIterator {
    type Item = Event;

    fn next(&mut self) -> Option<Self::Item> {
        let mut next_segment = self.frontier.pop()?;

        let next_event = next_segment.next();

        self.frontier.push(next_segment);

        next_event
    }
}

struct SortedSegment {
    reader: BufReader<File>,
    buffer: VecDeque<Event>,
    capacity: usize,
}

impl SortedSegment {
    fn new(path: &Path, capacity: usize) -> Result<Self> {
        let file = OpenOptions::new().read(true).write(false).open(path)?;

        let mut reader = BufReader::new(file);

        let mut buffer = VecDeque::with_capacity(capacity);

        for _ in 0..capacity {
            if let Ok(event) = bincode::deserialize_from(&mut reader) {
                buffer.push_back(event)
            } else {
                break;
            }
        }

        Ok(Self {
            reader,
            buffer,
            capacity,
        })
    }
}

impl Iterator for SortedSegment {
    type Item = Event;

    fn next(&mut self) -> Option<Self::Item> {
        let next_event = self.buffer.pop_front();

        if next_event.is_some() && self.buffer.is_empty() {
            for _ in 0..self.capacity {
                if let Ok(event) = bincode::deserialize_from(&mut self.reader) {
                    self.buffer.push_back(event)
                } else {
                    break;
                }
            }
        }

        next_event
    }
}

impl Ord for SortedSegment {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self.buffer.front(), other.buffer.front()) {
            (None, None) => Ordering::Equal,
            (None, _) => Ordering::Less,
            (_, None) => Ordering::Greater,
            (Some(this_event), Some(other_event)) => other_event.cmp(this_event),
        }
    }
}

impl PartialOrd for SortedSegment {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for SortedSegment {
    fn eq(&self, other: &Self) -> bool {
        self.buffer.front() == other.buffer.front()
    }
}

impl Eq for SortedSegment {}
