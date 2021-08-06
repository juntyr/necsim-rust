#![deny(clippy::pedantic)]
#![no_std]
#![feature(stmt_expr_attributes)]
#![feature(maybe_uninit_extra)]

extern crate alloc;

#[macro_use]
extern crate contracts;

pub mod cogs;
pub mod event_buffer;
pub mod value_buffer;

mod utils;
