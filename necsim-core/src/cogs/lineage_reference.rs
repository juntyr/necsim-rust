use core::hash::Hash;

use super::Habitat;

#[allow(clippy::module_name_repetitions)]
pub trait LineageReference<H: Habitat>: PartialEq + Eq + Hash + Clone {}
