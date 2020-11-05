#![deny(clippy::pedantic)]
#![no_std]

extern crate alloc;

#[cfg(feature = "cuda")]
#[macro_use]
extern crate necsim_cuda_derive;

#[cfg(feature = "cuda")]
#[macro_use]
extern crate rustacuda_derive;

#[macro_use]
extern crate contracts;

pub mod alias;
pub mod cogs;
pub mod r#f64;
