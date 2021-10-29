use std::{
    collections::VecDeque,
    convert::TryFrom,
    fs::{File, OpenOptions},
    io::BufReader,
    path::{Path, PathBuf},
};

use serde::Deserialize;

use necsim_core::lineage::Lineage;

use super::LineageFileHeader;

#[derive(Debug, Deserialize)]
#[serde(try_from = "LineageFileLoaderRaw")]
#[allow(clippy::module_name_repetitions)]
pub struct LineageFileLoader {
    header: LineageFileHeader,
    reader: BufReader<File>,
    buffer: VecDeque<Lineage>,
    capacity: usize,
}

impl LineageFileLoader {
    /// # Errors
    ///
    /// Fails if the `path` cannot be read as a list of lineages
    pub fn try_new(path: &Path, capacity: usize) -> anyhow::Result<Self> {
        let file = OpenOptions::new().read(true).write(false).open(path)?;
        let mut buf_reader = BufReader::new(file);

        let header: LineageFileHeader = bincode::deserialize_from(&mut buf_reader)?;

        let mut buffer = VecDeque::with_capacity(header.length.min(capacity));

        if let Ok(event) = bincode::deserialize_from(&mut buf_reader) {
            buffer.push_back(event);
        }

        Ok(Self {
            header,
            reader: buf_reader,
            buffer,
            capacity,
        })
    }
}

impl TryFrom<LineageFileLoaderRaw> for LineageFileLoader {
    type Error = anyhow::Error;

    fn try_from(raw: LineageFileLoaderRaw) -> Result<Self, Self::Error> {
        Self::try_new(&raw.path, raw.capacity)
    }
}

impl Iterator for LineageFileLoader {
    type Item = Lineage;

    fn next(&mut self) -> Option<Self::Item> {
        let next_lineage = self.buffer.pop_front();

        if next_lineage.is_some() {
            self.header.length -= 1;
        }

        if next_lineage.is_some() && self.buffer.is_empty() {
            for _ in 0..self.capacity {
                if let Ok(event) = bincode::deserialize_from(&mut self.reader) {
                    self.buffer.push_back(event);
                } else {
                    break;
                }
            }
        }

        next_lineage
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.header.length, Some(self.header.length))
    }
}

impl ExactSizeIterator for LineageFileLoader {}

#[derive(Debug, Deserialize)]
#[serde(rename = "LineageFileLoader")]
struct LineageFileLoaderRaw {
    path: PathBuf,
    #[serde(default = "default_lineage_file_stream_capacity")]
    capacity: usize,
}

fn default_lineage_file_stream_capacity() -> usize {
    100_000
}
