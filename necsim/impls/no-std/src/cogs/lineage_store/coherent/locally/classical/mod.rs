use core::{marker::PhantomData, ops::Index};

use hashbrown::hash_map::HashMap;
use slab::Slab;

use necsim_core::{
    cogs::{Backup, F64Core, Habitat, OriginSampler},
    landscape::IndexedLocation,
    lineage::Lineage,
};

use crate::cogs::lineage_reference::in_memory::InMemoryLineageReference;

mod store;

#[allow(clippy::module_name_repetitions)]
#[derive(Debug)]
pub struct ClassicalLineageStore<F: F64Core, H: Habitat<F>> {
    lineages_store: Slab<Lineage>,
    indexed_location_to_lineage_reference: HashMap<IndexedLocation, InMemoryLineageReference>,
    _marker: PhantomData<(F, H)>,
}

impl<F: F64Core, H: Habitat<F>> Index<InMemoryLineageReference> for ClassicalLineageStore<F, H> {
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

impl<'h, F: F64Core, H: 'h + Habitat<F>> ClassicalLineageStore<F, H> {
    #[must_use]
    pub fn new<O: OriginSampler<'h, F, Habitat = H>>(mut origin_sampler: O) -> Self {
        #[allow(clippy::cast_possible_truncation)]
        let lineages_amount_hint = origin_sampler.full_upper_bound_size_hint() as usize;

        let mut lineages_store = Slab::with_capacity(lineages_amount_hint);

        let mut indexed_location_to_lineage_reference =
            HashMap::with_capacity(lineages_amount_hint);

        while let Some(indexed_location) = origin_sampler.next() {
            let lineage = Lineage::new(indexed_location.clone(), origin_sampler.habitat());

            let local_reference = InMemoryLineageReference::from(lineages_store.insert(lineage));

            indexed_location_to_lineage_reference.insert(indexed_location, local_reference);
        }

        lineages_store.shrink_to_fit();

        Self {
            lineages_store,
            indexed_location_to_lineage_reference,
            _marker: PhantomData::<(F, H)>,
        }
    }
}

#[contract_trait]
impl<F: F64Core, H: Habitat<F>> Backup for ClassicalLineageStore<F, H> {
    unsafe fn backup_unchecked(&self) -> Self {
        Self {
            lineages_store: self.lineages_store.clone(),
            indexed_location_to_lineage_reference: self
                .indexed_location_to_lineage_reference
                .clone(),
            _marker: PhantomData::<(F, H)>,
        }
    }
}
