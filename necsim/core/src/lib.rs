#![deny(clippy::pedantic)]
#![no_std]
#![feature(core_intrinsics)]
#![allow(incomplete_features)]
#![feature(generic_associated_types)]

#[doc(hidden)]
pub extern crate alloc;

#[cfg(feature = "mpi")]
#[doc(hidden)]
extern crate rsmpi as mpi;

#[macro_use]
extern crate contracts;

#[macro_use]
extern crate typed_builder;

pub mod cogs;
pub mod event;
pub mod intrinsics;
pub mod landscape;
pub mod lineage;
pub mod reporter;
pub mod simulation;
