use core::{hash::Hash, num::NonZeroUsize};

use necsim_core::cogs::{Backup, Habitat, LineageReference, MathsCore};

// InMemoryLineageReference uses a NonZeroUsize internally to enable same-size
// Option optimisation

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug, TypeLayout)]
#[allow(clippy::module_name_repetitions)]
pub struct InMemoryLineageReference(NonZeroUsize);

#[contract_trait]
impl Backup for InMemoryLineageReference {
    unsafe fn backup_unchecked(&self) -> Self {
        Self(self.0)
    }
}

impl<M: MathsCore, H: Habitat<M>> LineageReference<M, H> for InMemoryLineageReference {}

impl From<usize> for InMemoryLineageReference {
    fn from(reference: usize) -> Self {
        Self(unsafe { NonZeroUsize::new_unchecked(reference + 1) })
    }
}

impl From<InMemoryLineageReference> for usize {
    fn from(reference: InMemoryLineageReference) -> Self {
        reference.0.get() - 1
    }
}
