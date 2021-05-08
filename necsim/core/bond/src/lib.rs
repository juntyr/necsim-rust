#![deny(clippy::pedantic)]
#![no_std]
#![feature(rustc_attrs)]
#![feature(total_cmp)]

mod non_negative_f64;
mod non_zero_one_u64;
mod partition;
mod positive_f64;
mod zero_excl_one_incl_f64;
mod zero_incl_one_incl_f64;

pub use non_negative_f64::NonNegativeF64;
pub use non_zero_one_u64::NonZeroOneU64;
pub use partition::Partition;
pub use positive_f64::PositiveF64;
pub use zero_excl_one_incl_f64::ZeroExclOneInclF64;
pub use zero_incl_one_incl_f64::ZeroInclOneInclF64;
