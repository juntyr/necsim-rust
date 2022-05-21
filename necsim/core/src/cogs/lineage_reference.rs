use core::hash::Hash;

use super::{Backup, Habitat, MathsCore};

#[allow(clippy::module_name_repetitions)]
pub trait LineageReference<M: MathsCore, H: Habitat<M>>:
    Backup + PartialEq + Eq + Hash + core::fmt::Debug
{
}
