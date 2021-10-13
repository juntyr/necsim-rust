use core::hash::Hash;

use super::{Habitat, MathsCore};

#[allow(clippy::module_name_repetitions)]
pub trait LineageReference<M: MathsCore, H: Habitat<M>>:
    crate::cogs::Backup + PartialEq + Eq + Hash + Clone + core::fmt::Debug
{
}
