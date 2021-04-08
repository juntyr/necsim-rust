#![deny(clippy::pedantic)]

pub static CORE_VERSION: &str = env!("CARGO_PKG_VERSION");
pub static RUSTC_VERSION: &str = env!("RUSTC_VERSION");

pub use erased_serde;
pub use serde;

pub mod combinator;
pub mod common;
pub mod export;
pub mod import;
