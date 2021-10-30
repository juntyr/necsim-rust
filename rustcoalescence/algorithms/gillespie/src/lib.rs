#![deny(clippy::pedantic)]
#![feature(never_type)]
#![allow(incomplete_features)]
#![feature(specialization)]

#[macro_use]
extern crate serde_derive_state;

mod arguments;

pub mod classical;
pub mod event_skipping;
