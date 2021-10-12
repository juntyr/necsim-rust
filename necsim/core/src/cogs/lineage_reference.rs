use core::hash::Hash;

use super::{Habitat, F64Core};

#[allow(clippy::module_name_repetitions)]
pub trait LineageReference<F: F64Core, H: Habitat<F>>:
    crate::cogs::Backup + PartialEq + Eq + Hash + Clone + core::fmt::Debug
{
}
