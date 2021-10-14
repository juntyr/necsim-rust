#![deny(clippy::pedantic)]
#![no_std]
#![feature(total_cmp)]
#![feature(const_fn_floating_point_arithmetic)]
#![feature(const_float_bits_conv)]
#![feature(const_float_classify)]
#![feature(const_trait_impl)]
#![feature(const_type_name)]
#![feature(const_raw_ptr_deref)]
#![feature(const_maybe_uninit_as_ptr)]
#![feature(const_ptr_offset_from)]
#![feature(const_mut_refs)]

#[macro_use]
extern crate const_type_layout;

mod closed_unit_f64;
mod non_negative_f64;
mod non_zero_one_u64;
mod partition;
mod positive_f64;
mod positive_unit_f64;

pub use closed_unit_f64::ClosedUnitF64;
pub use non_negative_f64::NonNegativeF64;
pub use non_zero_one_u64::NonZeroOneU64;
pub use partition::Partition;
pub use positive_f64::PositiveF64;
pub use positive_unit_f64::PositiveUnitF64;
