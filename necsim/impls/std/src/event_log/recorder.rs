// Based on https://docs.rs/extsort/0.4.2/src/extsort/sorter.rs.html

// Copyright 2018 Andre-Philippe Paquet
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::{
    convert::TryFrom,
    fmt,
    fs::{self, OpenOptions},
    io::BufWriter,
    num::NonZeroUsize,
    path::{Path, PathBuf},
};

use anyhow::{Error, Result};
use serde::{Deserialize, Serialize, Serializer};

use necsim_core::event::{DispersalEvent, PackedEvent, SpeciationEvent};

use super::EventLogHeader;

#[allow(clippy::module_name_repetitions)]
#[derive(Deserialize)]
#[serde(try_from = "EventLogRecorderRaw")]
pub struct EventLogRecorder {
    segment_capacity: NonZeroUsize,
    directory: PathBuf,
    segment_index: usize,
    buffer: Vec<PackedEvent>,

    record_speciation: bool,
    record_dispersal: bool,
}

impl TryFrom<EventLogRecorderRaw> for EventLogRecorder {
    type Error = Error;

    fn try_from(raw: EventLogRecorderRaw) -> Result<Self, Self::Error> {
        Self::try_new(&raw.directory, raw.capacity)
    }
}

impl Serialize for EventLogRecorder {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        EventLogRecorderRaw {
            directory: self.directory.clone(),
            capacity: self.segment_capacity,
        }
        .serialize(serializer)
    }
}

impl Drop for EventLogRecorder {
    fn drop(&mut self) {
        if self.buffer.is_empty() {
            // Try to remove the directory if it is empty
            std::mem::drop(fs::remove_dir(&self.directory));
        } else {
            std::mem::drop(self.sort_and_write_segment());
        }
    }
}

impl EventLogRecorder {
    /// # Errors
    ///
    /// Fails to construct iff `path` is not a writable directory.
    pub fn try_new(path: &Path, segment_capacity: NonZeroUsize) -> Result<Self> {
        fs::create_dir_all(path)?;

        let metadata = fs::metadata(path)?;

        if !metadata.is_dir() {
            return Err(anyhow::anyhow!("{:?} is not a directory.", path));
        }

        if metadata.permissions().readonly() {
            return Err(anyhow::anyhow!("{:?} is a read-only directory.", path));
        }

        Ok(Self {
            segment_capacity,
            directory: path.to_owned(),
            segment_index: 0_usize,
            buffer: Vec::with_capacity(segment_capacity.get()),

            record_speciation: false,
            record_dispersal: false,
        })
    }

    /// # Errors
    ///
    /// Fails to construct iff `path` is not a writable directory.
    pub fn r#move(mut self, path: &Path) -> Result<Self> {
        if !self.buffer.is_empty() {
            self.sort_and_write_segment()?;
        } else if !path.starts_with(&self.directory) {
            // Try to remove the directory if it is empty and the
            //  new path is not a child of the current directory
            std::mem::drop(fs::remove_dir(&self.directory));
        }

        fs::create_dir_all(path)?;

        let metadata = fs::metadata(path)?;

        if !metadata.is_dir() {
            return Err(anyhow::anyhow!("{:?} is not a directory.", path));
        }

        if metadata.permissions().readonly() {
            return Err(anyhow::anyhow!("{:?} is a read-only directory.", path));
        }

        self.directory = path.to_owned();

        Ok(self)
    }

    /// # Errors
    ///
    /// Fails to construct iff `path` is not an empty directory.
    pub fn assert_empty(self) -> Result<Self> {
        if fs::read_dir(&self.directory)?.next().is_some() {
            return Err(anyhow::anyhow!(
                "{:?} is not an empty directory.\n\nIf you are starting a new simulation, clean \
                 out the existing log.\nIf you are pausing or resuming a simulation, try \
                 appending a\n simulation-slice-specific postfix to your log path, and keep all\n \
                 these log-slices in the same parent directory for easy analysis.",
                &self.directory
            ));
        }

        Ok(self)
    }

    #[must_use]
    pub fn directory(&self) -> &Path {
        &self.directory
    }

    pub fn set_event_filter(&mut self, record_speciation: bool, record_dispersal: bool) {
        self.record_speciation = record_speciation;
        self.record_dispersal = record_dispersal;
    }

    pub fn record_speciation(&mut self, event: &SpeciationEvent) {
        self.record_speciation = true;

        self.buffer.push(event.clone().into());

        if self.buffer.len() >= self.segment_capacity.get() {
            std::mem::drop(self.sort_and_write_segment());
        }
    }

    pub fn record_dispersal(&mut self, event: &DispersalEvent) {
        self.record_dispersal = true;

        self.buffer.push(event.clone().into());

        if self.buffer.len() >= self.segment_capacity.get() {
            std::mem::drop(self.sort_and_write_segment());
        }
    }

    fn sort_and_write_segment(&mut self) -> Result<()> {
        self.buffer.sort();

        let segment_path = self.directory.join(format!("{}", self.segment_index));
        self.segment_index += 1;

        let segment_file = OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .open(segment_path)?;
        let mut buf_writer = BufWriter::new(segment_file);

        bincode::serialize_into(
            &mut buf_writer,
            &EventLogHeader::new(
                self.buffer[0].event_time(),
                self.buffer[self.buffer.len() - 1].event_time(),
                self.buffer.len(),
                self.record_speciation,
                self.record_dispersal,
            ),
        )?;

        for event in self.buffer.drain(0..) {
            bincode::serialize_into(&mut buf_writer, &event)?;
        }

        buf_writer.into_inner()?;

        Ok(())
    }
}

impl fmt::Debug for EventLogRecorder {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        struct EventBufferLen(usize);

        impl fmt::Debug for EventBufferLen {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "Vec<Event; {}>", self.0)
            }
        }

        f.debug_struct(stringify!(EventLogRecorder))
            .field("segment_capacity", &self.segment_capacity)
            .field("directory", &self.directory)
            .field("segment_index", &self.segment_index)
            .field("buffer", &EventBufferLen(self.buffer.len()))
            .finish_non_exhaustive()
    }
}

#[derive(Serialize, Deserialize)]
#[serde(rename = "EventLog")]
#[serde(deny_unknown_fields)]
struct EventLogRecorderRaw {
    directory: PathBuf,
    #[serde(default = "default_event_log_recorder_segment_capacity")]
    capacity: NonZeroUsize,
}

fn default_event_log_recorder_segment_capacity() -> NonZeroUsize {
    NonZeroUsize::new(1_000_000_usize).unwrap()
}
