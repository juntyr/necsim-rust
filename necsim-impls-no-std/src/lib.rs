#![deny(clippy::pedantic)]
#![no_std]

extern crate alloc;

#[cfg(not(target_os = "cuda"))]
#[macro_use]
extern crate necsim_cuda_derive;

#[macro_use]
extern crate contracts;

#[macro_use]
extern crate rustacuda_derive;

pub mod cogs;
