#![deny(clippy::pedantic)]

#[macro_use]
extern crate contracts;

#[macro_use]
extern crate necsim_core;

pub mod r#f64;

pub mod alias;
pub mod event_generator;
pub mod landscape;
pub mod reporter;
