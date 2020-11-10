#![deny(clippy::pedantic)]
#![no_std]
#![feature(stmt_expr_attributes)]

extern crate alloc;

#[macro_use]
extern crate contracts;

#[macro_use]
extern crate rust_cuda_derive;

pub mod cogs;
pub mod event_buffer;
