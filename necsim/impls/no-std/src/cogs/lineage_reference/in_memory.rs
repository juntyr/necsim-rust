use core::{hash::Hash, num::NonZeroUsize};

use necsim_core::cogs::{Habitat, LineageReference};

// InMemoryLineageReference uses a NonZeroUsize internally to enable same-size
// Option optimisation

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
#[allow(clippy::module_name_repetitions)]
pub struct InMemoryLineageReference(NonZeroUsize);

#[cfg(feature = "cuda")]
unsafe impl rustacuda_core::DeviceCopy for InMemoryLineageReference {}

impl<H: Habitat> LineageReference<H> for InMemoryLineageReference {}

impl From<usize> for InMemoryLineageReference {
    fn from(reference: usize) -> Self {
        Self(unsafe { NonZeroUsize::new_unchecked(reference + 1) })
    }
}

impl Into<usize> for InMemoryLineageReference {
    fn into(self) -> usize {
        self.0.get() - 1
    }
}
