use core::{marker::PhantomData, ops::Index};

use hashbrown::hash_map::HashMap;
use slab::Slab;

use necsim_core::{
    cogs::{Backup, F64Core, OriginSampler},
    landscape::Location,
    lineage::Lineage,
};

use crate::cogs::{
    habitat::almost_infinite::AlmostInfiniteHabitat,
    lineage_reference::in_memory::InMemoryLineageReference,
};

mod store;

#[allow(clippy::module_name_repetitions)]
#[derive(Debug)]
pub struct AlmostInfiniteLineageStore<F: F64Core> {
    lineages_store: Slab<Lineage>,
    location_to_lineage_reference: HashMap<Location, InMemoryLineageReference>,
    _marker: PhantomData<F>,
}

impl<F: F64Core> Index<InMemoryLineageReference> for AlmostInfiniteLineageStore<F> {
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

impl<F: F64Core> AlmostInfiniteLineageStore<F> {
    #[must_use]
    pub fn new<'h, O: OriginSampler<'h, F, Habitat = AlmostInfiniteHabitat<F>>>(
        mut origin_sampler: O,
    ) -> Self {
        #[allow(clippy::cast_possible_truncation)]
        let lineages_amount_hint = origin_sampler.full_upper_bound_size_hint() as usize;

        let mut lineages_store = Slab::with_capacity(lineages_amount_hint);
        let mut location_to_lineage_references = HashMap::with_capacity(lineages_amount_hint);

        while let Some(indexed_location) = origin_sampler.next() {
            location_to_lineage_references.insert(
                indexed_location.location().clone(),
                InMemoryLineageReference::from(lineages_store.len()),
            );

            let _local_reference =
                lineages_store.insert(Lineage::new(indexed_location, origin_sampler.habitat()));
        }

        lineages_store.shrink_to_fit();

        Self {
            lineages_store,
            location_to_lineage_reference: location_to_lineage_references,
            _marker: PhantomData::<F>,
        }
    }
}

#[contract_trait]
impl<F: F64Core> Backup for AlmostInfiniteLineageStore<F> {
    unsafe fn backup_unchecked(&self) -> Self {
        Self {
            lineages_store: self.lineages_store.clone(),
            location_to_lineage_reference: self.location_to_lineage_reference.clone(),
            _marker: PhantomData::<F>,
        }
    }
}
