use core::{marker::PhantomData, ops::Index};

use hashbrown::hash_map::HashMap;
use slab::Slab;

use necsim_core::{
    cogs::{Backup, MathsCore, OriginSampler},
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
pub struct AlmostInfiniteLineageStore<M: MathsCore> {
    lineages_store: Slab<Lineage>,
    location_to_lineage_reference: HashMap<Location, InMemoryLineageReference>,
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

impl<M: MathsCore> AlmostInfiniteLineageStore<M> {
    #[must_use]
    pub fn new<'h, O: OriginSampler<'h, M, Habitat = AlmostInfiniteHabitat<M>>>(
        origin_sampler: O,
    ) -> Self {
        #[allow(clippy::cast_possible_truncation)]
        let lineages_amount_hint = origin_sampler.full_upper_bound_size_hint() as usize;

        let mut lineages_store = Slab::with_capacity(lineages_amount_hint);
        let mut location_to_lineage_references = HashMap::with_capacity(lineages_amount_hint);

        for lineage in origin_sampler {
            location_to_lineage_references.insert(
                lineage.indexed_location.location().clone(),
                InMemoryLineageReference::from(lineages_store.len()),
            );

            let _local_reference = lineages_store.insert(lineage);
        }

        lineages_store.shrink_to_fit();

        Self {
            lineages_store,
            location_to_lineage_reference: location_to_lineage_references,
            _marker: PhantomData::<M>,
        }
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
