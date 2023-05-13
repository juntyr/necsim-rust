#![deny(clippy::pedantic)]
#![no_std]
#![feature(core_intrinsics)]
#![feature(const_trait_impl)]
#![feature(const_type_name)]
#![feature(const_mut_refs)]
#![feature(const_refs_to_cell)]
#![feature(generic_const_exprs)]
#![feature(arbitrary_self_types)]
#![cfg_attr(target_os = "cuda", feature(asm_experimental_arch))]
#![cfg_attr(target_os = "cuda", feature(stdsimd))]
#![allow(incomplete_features)]
#![feature(specialization)]

extern crate alloc;

#[macro_use]
extern crate const_type_layout;

#[macro_use]
extern crate contracts;

pub mod cogs;
pub mod event_buffer;
pub mod value_buffer;

mod utils;
