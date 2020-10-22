#![deny(clippy::pedantic)]

#[macro_use]
extern crate contracts;

#[macro_use]
extern crate typed_builder;

pub mod event_generator;
#[macro_use]
pub mod landscape;
pub mod build;
pub mod lineage;
pub mod reporter;
pub mod rng;
pub mod simulation;
//pub mod builder;
