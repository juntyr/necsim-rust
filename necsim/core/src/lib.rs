#![deny(clippy::pedantic)]
#![allow(clippy::type_repetition_in_bounds)]
#![no_std]
#![feature(const_trait_impl)]
#![feature(const_type_name)]
#![feature(const_mut_refs)]
#![feature(const_refs_to_cell)]
#![feature(control_flow_enum)]

#[doc(hidden)]
pub extern crate alloc;

#[macro_use]
extern crate const_type_layout;

#[macro_use]
extern crate contracts;

pub mod cogs;
pub mod event;
pub mod landscape;
pub mod lineage;
pub mod reporter;
pub mod simulation;
