#[cfg(not(target_os = "cuda"))]
pub mod host;

pub mod common;

#[cfg(target_os = "cuda")]
pub mod device;
