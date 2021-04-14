#![deny(clippy::pedantic)]
#![no_std]
#![feature(stmt_expr_attributes)]
#![allow(incomplete_features)]
#![feature(generic_associated_types)]
#![feature(total_cmp)]
#![feature(iter_advance_by)]
#![feature(fn_traits)]
#![feature(never_type)]
#![feature(debug_non_exhaustive)]
#![feature(option_result_unwrap_unchecked)]
#![feature(drain_filter)]

extern crate alloc;

#[cfg(feature = "cuda")]
#[macro_use]
extern crate rust_cuda_derive;

#[macro_use]
extern crate contracts;

pub mod alias;
pub mod cache;
pub mod cogs;
pub mod decomposition;
pub mod parallelisation;
pub mod partitioning;
pub mod reporter;
pub mod simulation;
