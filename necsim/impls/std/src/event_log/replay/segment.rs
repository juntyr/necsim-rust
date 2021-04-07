use std::{
    cmp::{Ord, Ordering},
    collections::VecDeque,
    fmt,
    fs::{File, OpenOptions},
    io::BufReader,
    path::Path,
};

use anyhow::Result;

use necsim_core::event::PackedEvent;

use crate::event_log::EventLogHeader;

#[allow(clippy::module_name_repetitions)]
pub struct SortedSegment {
    header: EventLogHeader,
    reader: BufReader<File>,
    buffer: VecDeque<PackedEvent>,
    capacity: usize,
}

impl fmt::Debug for SortedSegment {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_struct("SortedSegment")
            .field("header", &self.header)
            .field("capacity", &self.capacity)
            .field("next", &self.buffer.front())
            .finish()
    }
}

impl SortedSegment {
    pub fn new(path: &Path, capacity: usize) -> Result<Self> {
        let file = OpenOptions::new().read(true).write(false).open(path)?;

        let mut buf_reader = BufReader::new(file);

        let header: EventLogHeader = bincode::deserialize_from(&mut buf_reader)?;

        let mut buffer = VecDeque::with_capacity(capacity);

        for _ in 0..capacity {
            if let Ok(event) = bincode::deserialize_from(&mut buf_reader) {
                buffer.push_back(event)
            } else {
                break;
            }
        }

        Ok(Self {
            header,
            reader: buf_reader,
            buffer,
            capacity,
        })
    }

    pub fn header(&self) -> &EventLogHeader {
        &self.header
    }
}

impl Iterator for SortedSegment {
    type Item = PackedEvent;

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
