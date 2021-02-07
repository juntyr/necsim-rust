use std::path::Path;

use necsim_core::event::Event;

use anyhow::Result;
use commitlog::{message::MessageSet, CommitLog, LogOptions, Offset, ReadLimit};

#[allow(clippy::module_name_repetitions)]
pub enum EventReplayType {
    Disjoint(EventReplayIterator),
    Overlapping(EventReplayIterator),
}

#[allow(clippy::module_name_repetitions)]
pub struct EventReplayIterator {
    commit_log: CommitLog,
    position: Offset,
    buffer: Vec<Event>,
}

impl EventReplayIterator {
    pub(crate) const MAX_MESSAGE_SIZE: usize =
        std::mem::size_of::<Event>() * std::mem::size_of::<u64>();

    /// # Errors
    /// Fails to construct iff `commitlog` fails to open the log
    pub fn try_new(path: &Path) -> Result<EventReplayType> {
        // Initialise the CommitLog with a conservative message size upper bound
        let mut log_options = LogOptions::new(path);
        log_options.message_max_bytes(Self::MAX_MESSAGE_SIZE);

        let commit_log = CommitLog::new(log_options)?;

        let is_disjoint = commit_log.last_offset().map_or(true, |last_offset| {
            if last_offset > 0 {
                let first_event = commit_log
                    .read(0, ReadLimit::max_bytes(Self::MAX_MESSAGE_SIZE))
                    .ok()
                    .and_then(|messages| {
                        messages
                            .iter()
                            .next()
                            .map(|message| message.payload().to_owned())
                    });
                let last_event = commit_log
                    .read(last_offset, ReadLimit::max_bytes(Self::MAX_MESSAGE_SIZE))
                    .ok()
                    .and_then(|messages| {
                        messages
                            .iter()
                            .next()
                            .map(|message| message.payload().to_owned())
                    });

                first_event != last_event
            } else {
                true
            }
        });

        let iter = Self {
            commit_log,
            position: 0,
            buffer: Vec::new(),
        };

        Ok(if is_disjoint {
            EventReplayType::Disjoint(iter)
        } else {
            EventReplayType::Overlapping(iter)
        })
    }

    #[must_use]
    pub fn len(&self) -> u64 {
        self.commit_log.next_offset()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl Iterator for EventReplayIterator {
    type Item = Event;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(event) = self.buffer.pop() {
            return Some(event);
        }

        let log_section = self
            .commit_log
            .read(self.position, ReadLimit::default())
            .ok()?;

        self.buffer.reserve(log_section.len());

        for message in log_section.iter() {
            self.position = message.offset() + 1;

            if let Ok(event) = bincode::deserialize(message.payload()) {
                self.buffer.push(event);
            }
        }

        self.buffer.pop()
    }
}
