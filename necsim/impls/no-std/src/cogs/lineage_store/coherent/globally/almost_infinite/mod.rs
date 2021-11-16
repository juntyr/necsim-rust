use core::{marker::PhantomData, ops::Index};

use fnv::FnvBuildHasher;
use hashbrown::hash_map::HashMap;
use slab::Slab;

use necsim_core::{
    cogs::{Backup, MathsCore},
    landscape::Location,
    lineage::Lineage,
};

use crate::cogs::lineage_reference::in_memory::InMemoryLineageReference;

mod store;

#[allow(clippy::module_name_repetitions)]
#[derive(Debug)]
pub struct AlmostInfiniteLineageStore<M: MathsCore> {
    lineages_store: Slab<Lineage>,
    location_to_lineage_reference: HashMap<Location, InMemoryLineageReference, FnvBuildHasher>,
    _marker: PhantomData<M>,
}

impl<M: MathsCore> Index<InMemoryLineageReference> for AlmostInfiniteLineageStore<M> {
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
impl<M: MathsCore> Backup for AlmostInfiniteLineageStore<M> {
    unsafe fn backup_unchecked(&self) -> Self {
        Self {
            lineages_store: self.lineages_store.clone(),
            location_to_lineage_reference: self.location_to_lineage_reference.clone(),
            _marker: PhantomData::<M>,
        }
    }
}
