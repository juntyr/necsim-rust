#![deny(clippy::pedantic)]
#![no_std]

extern crate alloc;

#[macro_use]
extern crate necsim_cuda_derive;

#[macro_use]
extern crate contracts;

pub mod cogs;
