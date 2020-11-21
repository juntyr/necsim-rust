use core::{
    hash::Hash,
    iter::{ExactSizeIterator, Iterator},
};

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

#[allow(clippy::module_name_repetitions)]
pub struct InMemoryLineageReferenceIterator {
    from: usize,
    len: usize,
}

impl From<usize> for InMemoryLineageReferenceIterator {
    fn from(len: usize) -> Self {
        Self { from: 0_usize, len }
    }
}

impl Iterator for InMemoryLineageReferenceIterator {
    type Item = InMemoryLineageReference;

    fn next(&mut self) -> Option<Self::Item> {
        if self.from < self.len {
            self.from += 1;

            Some(InMemoryLineageReference::from(self.from - 1))
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len - self.from, Some(self.len - self.from))
    }
}

impl ExactSizeIterator for InMemoryLineageReferenceIterator {}
