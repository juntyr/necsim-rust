#![deny(clippy::pedantic)]
#![no_std]
#![feature(stmt_expr_attributes)]
#![feature(min_const_generics)]
#![feature(associated_type_bounds)]

extern crate alloc;

#[cfg_attr(target_os = "cuda", macro_use)]
extern crate contracts;

#[macro_use]
extern crate rust_cuda_derive;

pub mod cogs;
pub mod event_buffer;
pub mod exchange_buffer;
pub mod task_list;
pub mod value_buffer;
