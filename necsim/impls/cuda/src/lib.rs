#![deny(clippy::pedantic)]
#![no_std]
#![feature(stmt_expr_attributes)]

extern crate alloc;

#[macro_use]
extern crate contracts;

pub mod cogs;
pub mod event_buffer;
pub mod value_buffer;
