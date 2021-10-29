use serde::{Deserialize, Serialize};

pub mod loader;

#[derive(Debug, Deserialize, Serialize)]
struct LineageFileHeader {
    length: usize,
}
