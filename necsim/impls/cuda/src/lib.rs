#![deny(clippy::pedantic)]
#![no_std]
#![feature(maybe_uninit_extra)]
#![feature(core_intrinsics)]
#![cfg_attr(target_os = "cuda", feature(asm))]
#![cfg_attr(target_os = "cuda", feature(const_float_bits_conv))]

extern crate alloc;

#[macro_use]
extern crate contracts;

pub mod cogs;
pub mod event_buffer;
pub mod value_buffer;

mod utils;
