#![deny(clippy::pedantic)]
#![no_std]
#![feature(const_trait_impl)]
#![feature(const_type_name)]
#![feature(const_raw_ptr_deref)]
#![feature(const_maybe_uninit_as_ptr)]
#![feature(const_ptr_offset_from)]
#![feature(const_mut_refs)]
#![feature(const_fn_trait_bound)]
#![feature(const_refs_to_cell)]
#![allow(incomplete_features)]
#![feature(generic_associated_types)]

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
