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
    path::{Path, PathBuf},
};

use anyhow::{Error, Result};

use necsim_core::event::{DispersalEvent, PackedEvent, SpeciationEvent};

use super::EventLogHeader;

#[allow(clippy::module_name_repetitions)]
#[derive(serde::Deserialize)]
#[serde(try_from = "PathBuf")]
pub struct EventLogRecorder {
    segment_size: usize,
    directory: PathBuf,
    segment_index: usize,
    buffer: Vec<PackedEvent>,

    record_speciation: bool,
    record_dispersal: bool,
}

impl TryFrom<PathBuf> for EventLogRecorder {
    type Error = Error;

    fn try_from(path: PathBuf) -> Result<Self, Self::Error> {
        Self::try_new(&path)
    }
}

impl Drop for EventLogRecorder {
    fn drop(&mut self) {
        if !self.buffer.is_empty() {
            std::mem::drop(self.sort_and_write_segment());
        }
    }
}

impl EventLogRecorder {
    /// # Errors
    /// Fails to construct iff `path` is not a writable directory.
    pub fn try_new(path: &Path) -> Result<Self> {
        if !path.exists() {
            fs::create_dir_all(path)?;
        }

        let metadata = fs::metadata(path)?;

        if !metadata.is_dir() {
            return Err(anyhow::anyhow!("{:?} is not a directory.", path));
        }

        if metadata.permissions().readonly() {
            return Err(anyhow::anyhow!("{:?} is read-only.", path));
        }

        let segment_size = 1_000_000_usize;

        Ok(Self {
            segment_size,
            directory: path.to_owned(),
            segment_index: 0_usize,
            buffer: Vec::with_capacity(segment_size),

            record_speciation: false,
            record_dispersal: false,
        })
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

        if self.buffer.len() >= self.segment_size {
            std::mem::drop(self.sort_and_write_segment());
        }
    }

    pub fn record_dispersal(&mut self, event: &DispersalEvent) {
        self.record_dispersal = true;

        self.buffer.push(event.clone().into());

        if self.buffer.len() >= self.segment_size {
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
            .open(&segment_path)?;
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

        f.debug_struct("EventLogRecorder")
            .field("segment_size", &self.segment_size)
            .field("directory", &self.directory)
            .field("segment_index", &self.segment_index)
            .field("buffer", &EventBufferLen(self.buffer.len()))
            .finish()
    }
}
