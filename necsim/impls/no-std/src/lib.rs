#![deny(clippy::pedantic)]
#![no_std]
#![feature(stmt_expr_attributes)]
#![allow(incomplete_features)]
#![feature(generic_associated_types)]

extern crate alloc;

#[cfg(feature = "cuda")]
#[macro_use]
extern crate rust_cuda_derive;

#[macro_use]
extern crate contracts;

pub mod alias;
pub mod cogs;
pub mod r#f64;
pub mod reporter;
pub mod simulation;
