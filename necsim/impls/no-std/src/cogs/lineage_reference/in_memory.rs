use core::{hash::Hash, num::NonZeroUsize};

use necsim_core::cogs::{Backup, F64Core, Habitat, LineageReference};

// InMemoryLineageReference uses a NonZeroUsize internally to enable same-size
// Option optimisation

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
#[allow(clippy::module_name_repetitions)]
pub struct InMemoryLineageReference(NonZeroUsize);

#[contract_trait]
impl Backup for InMemoryLineageReference {
    unsafe fn backup_unchecked(&self) -> Self {
        Self(self.0)
    }
}

impl<F: F64Core, H: Habitat<F>> LineageReference<F, H> for InMemoryLineageReference {}

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
