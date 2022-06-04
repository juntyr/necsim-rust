use core::hash::Hash;

use necsim_core::cogs::{Backup, Habitat, LineageReference, MathsCore};

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug, TypeLayout)]
#[allow(clippy::module_name_repetitions)]
pub struct InMemoryLineageReference(usize);

#[contract_trait]
impl Backup for InMemoryLineageReference {
    unsafe fn backup_unchecked(&self) -> Self {
        Self(self.0)
    }
}

impl<M: MathsCore, H: Habitat<M>> LineageReference<M, H> for InMemoryLineageReference {}

impl From<usize> for InMemoryLineageReference {
    fn from(reference: usize) -> Self {
        Self(reference)
    }
}

impl From<InMemoryLineageReference> for usize {
    fn from(reference: InMemoryLineageReference) -> Self {
        reference.0
    }
}
