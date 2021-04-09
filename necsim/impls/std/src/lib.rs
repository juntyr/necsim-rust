#![deny(clippy::pedantic)]
#![allow(incomplete_features)]
#![feature(generic_associated_types)]
#![feature(total_cmp)]

#[macro_use]
extern crate contracts;

pub mod bounded;
pub mod cogs;
pub mod event_log;
pub mod partitioning;
