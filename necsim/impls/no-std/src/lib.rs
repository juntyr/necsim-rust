#![deny(clippy::pedantic)]
#![no_std]
#![feature(total_cmp)]
#![feature(iter_advance_by)]
#![feature(option_result_unwrap_unchecked)]
#![feature(drain_filter)]
#![feature(type_alias_impl_trait)]
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
#![feature(specialization)]

extern crate alloc;

#[macro_use]
extern crate const_type_layout;

#[macro_use]
extern crate contracts;

#[macro_use]
extern crate log;

pub mod alias;
pub mod array2d;
pub mod cache;
pub mod cogs;
pub mod decomposition;
pub mod parallelisation;
