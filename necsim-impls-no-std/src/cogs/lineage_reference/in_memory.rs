use core::hash::Hash;

use necsim_core::cogs::{Habitat, LineageReference};

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
#[allow(clippy::module_name_repetitions)]
#[cfg_attr(feature = "cuda", derive(DeviceCopy))]
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
