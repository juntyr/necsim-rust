use core::hash::Hash;

use necsim_core::cogs::{Habitat, LineageReference};

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
#[allow(clippy::module_name_repetitions)]
pub struct IndependentLineageReference(());

impl IndependentLineageReference {
    pub(super) fn new() -> Self {
        Self(())
    }
}

#[cfg(feature = "cuda")]
unsafe impl rustacuda_core::DeviceCopy for IndependentLineageReference {}

impl<H: Habitat> LineageReference<H> for IndependentLineageReference {}
