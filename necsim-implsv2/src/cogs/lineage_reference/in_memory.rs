use std::hash::Hash;

use necsim_corev2::cogs::{Habitat, LineageReference};

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
#[allow(clippy::module_name_repetitions)]
pub struct InMemoryLineageReference(usize);

impl<H: Habitat> LineageReference<H> for InMemoryLineageReference {}

impl From<usize> for InMemoryLineageReference {
    fn from(reference: usize) -> Self {
        Self(reference)
    }
}

impl Into<usize> for InMemoryLineageReference {
    fn into(self) -> usize {
        self.0
    }
}
