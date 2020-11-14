#![deny(clippy::pedantic)]
#![allow(incomplete_features)]
#![feature(generic_associated_types)]

#[macro_use]
extern crate contracts;

pub mod cogs;
pub mod reporter;
pub mod simulation;
