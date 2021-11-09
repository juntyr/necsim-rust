use core::{marker::PhantomData, ops::Index};

use alloc::vec::Vec;

use fnv::FnvBuildHasher;
use hashbrown::hash_map::HashMap;
use slab::Slab;

use necsim_core::{
    cogs::{Backup, Habitat, MathsCore, OriginSampler},
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

impl<'h, M: MathsCore, H: 'h + Habitat<M>> GillespieLineageStore<M, H> {
    #[must_use]
    pub fn new<O: OriginSampler<'h, M, Habitat = H>>(origin_sampler: O) -> Self {
        #[allow(clippy::cast_possible_truncation)]
        let lineages_amount_hint = origin_sampler.full_upper_bound_size_hint() as usize;

        let mut lineages_store = Slab::with_capacity(lineages_amount_hint);

        let landscape_extent = origin_sampler.habitat().get_extent();

        let mut location_to_lineage_references = Array2D::filled_with(
            Vec::new(),
            landscape_extent.height() as usize,
            landscape_extent.width() as usize,
        );

        let mut indexed_location_to_lineage_reference =
            HashMap::with_capacity_and_hasher(lineages_amount_hint, FnvBuildHasher::default());

        let x_from = landscape_extent.x();
        let y_from = landscape_extent.y();

        for lineage in origin_sampler {
            let x_offset = lineage.indexed_location.location().x() - x_from;
            let y_offset = lineage.indexed_location.location().y() - y_from;

            let lineages_at_location =
                &mut location_to_lineage_references[(y_offset as usize, x_offset as usize)];

            // Insert the global lineage reference into the per-indexed-location
            //  lookup
            indexed_location_to_lineage_reference.insert(
                lineage.indexed_location.clone(),
                (lineage.global_reference.clone(), lineages_at_location.len()),
            );

            // Insert the local lineage reference into the per-location lookup
            let local_reference = InMemoryLineageReference::from(lineages_store.insert(lineage));
            lineages_at_location.push(local_reference);
        }

        lineages_store.shrink_to_fit();

        Self {
            lineages_store,
            location_to_lineage_references,
            indexed_location_to_lineage_reference,
            _marker: PhantomData::<(M, H)>,
        }
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
