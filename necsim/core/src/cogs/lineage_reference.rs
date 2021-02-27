use core::hash::Hash;

use super::Habitat;

#[allow(clippy::module_name_repetitions)]
pub trait LineageReference<H: Habitat>:
    crate::cogs::Backup + PartialEq + Eq + Hash + Clone + core::fmt::Debug
{
}
