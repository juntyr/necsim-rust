//! This module re-exports a number of commonly-used types for working with
//! RustaCUDA.
//!
//! This allows the user to `use rustacuda::prelude::*;` and have the most
//! commonly-used types available quickly.

pub use crate::{
    context::{Context, ContextFlags},
    device::Device,
    memory::{CopyDestination, DeviceBuffer, UnifiedBuffer},
    module::Module,
    stream::{Stream, StreamFlags},
    CudaFlags,
};
