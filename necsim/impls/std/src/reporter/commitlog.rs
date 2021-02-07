use std::{io::Cursor, path::Path};

use necsim_core::{
    event::Event,
    reporter::{EventFilter, Reporter},
};

use anyhow::Result;
use commitlog::{CommitLog, LogOptions};

#[allow(clippy::module_name_repetitions)]
pub struct CommitLogReporter {
    commit_log: CommitLog,
    buffer: Vec<u8>,
}

impl Drop for CommitLogReporter {
    fn drop(&mut self) {
        let _ = self.commit_log.flush();
    }
}

impl EventFilter for CommitLogReporter {
    const REPORT_DISPERSAL: bool = true;
    const REPORT_SPECIATION: bool = true;
}

impl Reporter for CommitLogReporter {
    fn report_event(&mut self, event: &Event) {
        let mut cursor: Cursor<&mut Vec<u8>> = Cursor::new(&mut self.buffer);

        if let Ok(()) = bincode::serialize_into(&mut cursor, event) {
            #[allow(clippy::cast_possible_truncation)]
            let serialised_size = cursor.position() as usize;

            let _ = self.commit_log.append_msg(&self.buffer[..serialised_size]);
        }
    }
}

impl CommitLogReporter {
    /// # Errors
    /// Fails to construct iff `bincode` cannot compute the size of an event or
    ///  `commitlog` fails to open the log
    pub fn try_new(path: &Path) -> Result<Self> {
        let event_size = std::mem::size_of::<Event>();

        // Initialise the CommitLog with a conservative message size upper bound
        let mut log_options = LogOptions::new(path);
        log_options.message_max_bytes(event_size * std::mem::size_of::<u64>());

        let mut commit_log = CommitLog::new(log_options)?;

        if commit_log.last_offset().is_some() {
            commit_log.truncate(0)?;
        }

        Ok(Self {
            commit_log,
            buffer: vec![0; event_size],
        })
    }
}
