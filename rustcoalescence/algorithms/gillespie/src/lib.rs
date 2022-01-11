#![deny(clippy::pedantic)]
#![feature(never_type)]
#![feature(generic_associated_types)]
#![allow(incomplete_features)]
#![feature(specialization)]

#[macro_use]
extern crate serde_derive_state;

mod arguments;

pub mod event_skipping;
pub mod gillespie;
