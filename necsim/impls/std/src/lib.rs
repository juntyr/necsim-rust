#![deny(clippy::pedantic)]
#![allow(incomplete_features)]
#![feature(generic_associated_types)]
#![feature(total_cmp)]
#![feature(maybe_uninit_ref)]

#[macro_use]
extern crate contracts;

pub mod cogs;
pub mod reporter;
