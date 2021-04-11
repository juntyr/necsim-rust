#![deny(clippy::pedantic)]
#![allow(incomplete_features)]
#![feature(generic_associated_types)]
#![feature(total_cmp)]
#![feature(never_type)]
#![feature(associated_type_bounds)]

#[macro_use]
extern crate contracts;

#[macro_use]
extern crate log;

pub mod algorithm;
pub mod bounded;
pub mod cogs;
pub mod event_log;
pub mod partitioning;
pub mod scenario;
