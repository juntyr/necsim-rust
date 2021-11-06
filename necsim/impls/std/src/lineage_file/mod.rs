use serde::{Deserialize, Serialize};

pub mod loader;
pub mod saver;

#[derive(Debug, Deserialize, Serialize)]
struct LineageFileHeader {
    length: usize,
}
