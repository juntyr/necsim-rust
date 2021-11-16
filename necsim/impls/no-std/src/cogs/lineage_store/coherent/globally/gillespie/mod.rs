use core::{marker::PhantomData, ops::Index};

use alloc::vec::Vec;

use fnv::FnvBuildHasher;
use hashbrown::hash_map::HashMap;
use slab::Slab;

use necsim_core::{
    cogs::{Backup, Habitat, MathsCore},
    landscape::IndexedLocation,
    lineage::{GlobalLineageReference, Lineage},
};

use crate::{array2d::Array2D, cogs::lineage_reference::in_memory::InMemoryLineageReference};

mod store;

#[allow(clippy::module_name_repetitions)]
#[derive(Debug)]
pub struct GillespieLineageStore<M: MathsCore, H: Habitat<M>> {
    lineages_store: Slab<Lineage>,
    location_to_lineage_references: Array2D<Vec<InMemoryLineageReference>>,
    indexed_location_to_lineage_reference:
        HashMap<IndexedLocation, (GlobalLineageReference, usize), FnvBuildHasher>,
    _marker: PhantomData<(M, H)>,
}

impl<M: MathsCore, H: Habitat<M>> Index<InMemoryLineageReference> for GillespieLineageStore<M, H> {
    type Output = Lineage;

    #[must_use]
    #[debug_requires(
        self.lineages_store.contains(reference.into()),
        "lineage reference is valid in the lineage store"
    )]
    fn index(&self, reference: InMemoryLineageReference) -> &Self::Output {
        &self.lineages_store[usize::from(reference)]
    }
}

#[contract_trait]
impl<M: MathsCore, H: Habitat<M>> Backup for GillespieLineageStore<M, H> {
    unsafe fn backup_unchecked(&self) -> Self {
        Self {
            lineages_store: self.lineages_store.clone(),
            location_to_lineage_references: self.location_to_lineage_references.clone(),
            indexed_location_to_lineage_reference: self
                .indexed_location_to_lineage_reference
                .clone(),
            _marker: PhantomData::<(M, H)>,
        }
    }
}
