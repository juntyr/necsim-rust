#![deny(clippy::pedantic)]
#![no_std]
#![feature(total_cmp)]
#![feature(iter_advance_by)]
#![feature(option_result_unwrap_unchecked)]
#![feature(drain_filter)]
#![feature(min_type_alias_impl_trait)]
#![allow(incomplete_features)]
#![feature(generic_associated_types)]
#![feature(specialization)]

extern crate alloc;

#[macro_use]
extern crate contracts;

#[macro_use]
extern crate log;

pub mod alias;
pub mod array2d;
pub mod cache;
pub mod cogs;
pub mod decomposition;
pub mod parallelisation;
