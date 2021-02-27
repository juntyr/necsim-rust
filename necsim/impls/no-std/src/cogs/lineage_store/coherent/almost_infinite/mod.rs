use core::ops::Index;

use hashbrown::hash_map::HashMap;
use slab::Slab;

use necsim_core::{
    cogs::{Backup, OriginSampler},
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
pub struct CoherentAlmostInfiniteLineageStore {
    lineages_store: Slab<Lineage>,
    location_to_lineage_references: HashMap<Location, InMemoryLineageReference>,
}

impl Index<InMemoryLineageReference> for CoherentAlmostInfiniteLineageStore {
    type Output = Lineage;

    #[must_use]
    #[debug_requires(
        Into::<usize>::into(reference) < self.lineages_store.len(),
        "lineage reference is in range"
    )]
    fn index(&self, reference: InMemoryLineageReference) -> &Self::Output {
        &self.lineages_store[Into::<usize>::into(reference)]
    }
}

impl CoherentAlmostInfiniteLineageStore {
    #[must_use]
    pub fn new<'h, O: OriginSampler<'h, Habitat = AlmostInfiniteHabitat>>(
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
            location_to_lineage_references,
        }
    }
}

#[contract_trait]
impl Backup for CoherentAlmostInfiniteLineageStore {
    unsafe fn backup_unchecked(&self) -> Self {
        Self {
            lineages_store: self.lineages_store.clone(),
            location_to_lineage_references: self.location_to_lineage_references.clone(),
        }
    }
}
