use std::{
    cmp::{Ord, Ordering},
    collections::BinaryHeap,
    fs::{File, OpenOptions},
    io::BufReader,
    path::{Path, PathBuf},
};

use necsim_core::event::Event;

use anyhow::Result;

// TODO: Make merging more efficient

#[allow(clippy::module_name_repetitions)]
pub struct EventReplayIterator {
    frontier: BinaryHeap<SortedSegment>,
}

impl EventReplayIterator {
    /// # Errors
    ///
    /// Returns `Err` iff any of the paths could not be read.
    pub fn try_new(paths: &[PathBuf]) -> Result<Self> {
        let mut frontier = BinaryHeap::with_capacity(paths.len());

        for path in paths {
            frontier.push(SortedSegment::new(path)?);
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
    front: Option<Event>,
    reader: BufReader<File>,
}

impl SortedSegment {
    fn new(path: &Path) -> Result<Self> {
        let file = OpenOptions::new().read(true).write(false).open(path)?;

        let mut reader = BufReader::new(file);

        let front = bincode::deserialize_from(&mut reader).ok();

        Ok(Self { front, reader })
    }
}

impl Iterator for SortedSegment {
    type Item = Event;

    fn next(&mut self) -> Option<Self::Item> {
        let next_event = self.front.take();

        if next_event.is_some() {
            self.front = bincode::deserialize_from(&mut self.reader).ok();
        }

        next_event
    }
}

impl Ord for SortedSegment {
    fn cmp(&self, other: &Self) -> Ordering {
        match (&self.front, &other.front) {
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
        self.front == other.front
    }
}

impl Eq for SortedSegment {}
