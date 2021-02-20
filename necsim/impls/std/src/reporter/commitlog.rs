use std::{io::Cursor, path::Path};

use necsim_core::{
    event::Event,
    reporter::{EventFilter, Reporter},
};

use anyhow::Result;
use commitlog::{message::MessageSet, CommitLog, LogOptions, ReadLimit};

#[allow(clippy::module_name_repetitions)]
pub struct CommitLogReporter {
    commit_log: CommitLog,
    buffer: Box<[u8]>,
    is_disjoint: bool,
}

impl Drop for CommitLogReporter {
    fn drop(&mut self) {
        // Mark a log as non-disjoint by copying the first message such that
        //  the log starts and ends with the same message
        if !self.is_disjoint && self.commit_log.last_offset().is_some() {
            if let Ok(messages) = self
                .commit_log
                .read(0, ReadLimit::max_bytes(Self::MAX_MESSAGE_SIZE))
            {
                if let Some(first_message) = messages.iter().next() {
                    let _ = self.commit_log.append_msg(first_message.payload());
                }
            }
        }

        let _ = self.commit_log.flush();
    }
}

impl EventFilter for CommitLogReporter {
    const REPORT_DISPERSAL: bool = true;
    const REPORT_SPECIATION: bool = true;
}

impl Reporter for CommitLogReporter {
    fn report_event(&mut self, event: &Event) {
        let mut cursor: Cursor<&mut [u8]> = Cursor::new(self.buffer.as_mut());

        if let Ok(()) = bincode::serialize_into(&mut cursor, event) {
            #[allow(clippy::cast_possible_truncation)]
            let serialised_size = cursor.position() as usize;

            let _ = self.commit_log.append_msg(&self.buffer[..serialised_size]);
        }
    }
}

impl CommitLogReporter {
    const MAX_MESSAGE_SIZE: usize = crate::event_replay::EventReplayIterator::MAX_MESSAGE_SIZE;

    /// # Errors
    /// Fails to construct iff `commitlog` fails to open the log
    pub fn try_new(path: &Path) -> Result<Self> {
        // Initialise the CommitLog with a conservative message size upper bound
        let mut log_options = LogOptions::new(path);
        log_options.message_max_bytes(Self::MAX_MESSAGE_SIZE);

        let mut commit_log = CommitLog::new(log_options)?;

        if commit_log.last_offset().is_some() {
            commit_log.truncate(0)?;
        }

        Ok(Self {
            commit_log,
            buffer: vec![0; Self::MAX_MESSAGE_SIZE].into_boxed_slice(),
            is_disjoint: false,
        })
    }

    // TODO: This will no longer be required in the custom improved event log
    pub fn mark_disjoint(&mut self) {
        self.is_disjoint = true;
    }
}
