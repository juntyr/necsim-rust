#![deny(clippy::pedantic)]

pub static CORE_VERSION: &str = env!("CARGO_PKG_VERSION");
pub static RUSTC_VERSION: &str = env!("RUSTC_VERSION");

#[cfg(feature = "export")]
#[doc(hidden)]
pub use erased_serde;
#[cfg(feature = "export")]
#[doc(hidden)]
pub use serde;

#[cfg(feature = "export")]
pub mod export;
#[cfg(all(feature = "import", not(feature = "export")))]
mod export;
#[cfg(feature = "import")]
pub mod import;
