#![deny(clippy::pedantic)]
#![no_std]
#![feature(stmt_expr_attributes)]
#![allow(incomplete_features)]
#![feature(generic_associated_types)]
#![feature(total_cmp)]
#![feature(iter_advance_by)]
#![feature(fn_traits)]
#![feature(never_type)]
#![feature(option_result_unwrap_unchecked)]
#![feature(drain_filter)]
#![feature(specialization)]

extern crate alloc;

#[macro_use]
extern crate contracts;

#[macro_use]
extern crate log;

pub mod alias;
pub mod cache;
pub mod cogs;
pub mod decomposition;
pub mod parallelisation;
