#![deny(clippy::pedantic)]
#![no_std]
#![feature(stmt_expr_attributes)]
#![feature(min_const_generics)]

extern crate alloc;

#[cfg_attr(target_os = "cuda", macro_use)]
extern crate contracts;

#[macro_use]
extern crate rust_cuda_derive;

pub mod cogs;
pub mod event_buffer;
pub mod value_buffer;
