#![deny(clippy::pedantic)]
#![no_std]
#![feature(const_type_name)]
#![feature(offset_of)]
#![feature(control_flow_enum)]
#![feature(min_specialization)]

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
