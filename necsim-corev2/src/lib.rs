#![deny(clippy::pedantic)]

#[macro_use]
extern crate contracts;

#[macro_use]
extern crate typed_builder;

pub mod cogs;
pub mod event;
pub mod landscape;
pub mod lineage;
pub mod reporter;
pub mod rng;
pub mod simulation;

mod test;
