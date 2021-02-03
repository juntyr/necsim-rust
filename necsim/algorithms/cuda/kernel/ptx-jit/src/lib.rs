#![deny(clippy::pedantic)]
#![cfg_attr(target_os = "cuda", no_std)]

#[cfg(not(target_os = "cuda"))]
pub mod host;

#[cfg(target_os = "cuda")]
pub mod device;
