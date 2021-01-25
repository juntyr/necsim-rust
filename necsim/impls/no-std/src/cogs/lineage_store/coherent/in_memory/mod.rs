use core::{marker::PhantomData, ops::Index};

use alloc::vec::Vec;

use array2d::Array2D;
use hashbrown::hash_map::HashMap;

use necsim_core::{
    cogs::{Habitat, LineageStore},
    intrinsics::floor,
    landscape::{IndexedLocation, Location},
    lineage::{GlobalLineageReference, Lineage},
};

use crate::cogs::lineage_reference::in_memory::InMemoryLineageReference;

mod store;

#[allow(clippy::module_name_repetitions)]
#[derive(Debug)]
pub struct CoherentInMemoryLineageStore<H: Habitat> {
    lineages_store: Vec<Lineage>,
    location_to_lineage_references: Array2D<Vec<InMemoryLineageReference>>,
    indexed_location_to_lineage_reference:
        HashMap<IndexedLocation, (GlobalLineageReference, usize)>,
    _marker: PhantomData<H>,
}

impl<H: Habitat> Index<InMemoryLineageReference> for CoherentInMemoryLineageStore<H> {
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

impl<H: Habitat> CoherentInMemoryLineageStore<H> {
    #[must_use]
    #[allow(clippy::float_cmp)]
    #[debug_ensures(if sample_percentage == 0.0_f64 {
        ret.get_number_total_lineages() == 0
    } else if sample_percentage == 1.0_f64 {
        ret.get_number_total_lineages() as u64 == habitat.get_total_habitat()
    } else {
        true
    }, "samples active lineages according to sample_percentage")]
    pub fn new(sample_percentage: f64, habitat: &H) -> Self {
        #[allow(clippy::cast_possible_truncation)]
        #[allow(clippy::cast_sign_loss)]
        #[allow(clippy::cast_precision_loss)]
        let total_number_of_lineages =
            ((habitat.get_total_habitat() as f64) * sample_percentage) as usize;

        #[allow(clippy::cast_possible_truncation)]
        #[allow(clippy::cast_sign_loss)]
        #[allow(clippy::cast_precision_loss)]
        let mut lineages_store = Vec::with_capacity(total_number_of_lineages);

        let landscape_extent = habitat.get_extent();

        let mut location_to_lineage_references = Array2D::filled_with(
            Vec::new(),
            landscape_extent.height() as usize,
            landscape_extent.width() as usize,
        );

        let mut indexed_location_to_lineage_reference =
            HashMap::with_capacity(total_number_of_lineages);

        let mut extra_indexed_locations: Vec<(f64, IndexedLocation)> = Vec::new();

        let x_from = landscape_extent.x();
        let y_from = landscape_extent.y();

        for y_offset in 0..landscape_extent.height() {
            for x_offset in 0..landscape_extent.width() {
                let location = Location::new(x_from + x_offset, y_from + y_offset);

                let lineages_at_location =
                    &mut location_to_lineage_references[(y_offset as usize, x_offset as usize)];

                let sampled_habitat_at_location_max =
                    f64::from(habitat.get_habitat_at_location(&location)) * sample_percentage;

                #[allow(clippy::cast_possible_truncation)]
                #[allow(clippy::cast_sign_loss)]
                let sampled_habitat_at_location = floor(sampled_habitat_at_location_max) as u32;

                for index_at_location in 0..sampled_habitat_at_location {
                    let indexed_location =
                        IndexedLocation::new(location.clone(), index_at_location);

                    let lineage = Lineage::new(indexed_location.clone(), habitat);
                    let local_reference = InMemoryLineageReference::from(lineages_store.len());

                    indexed_location_to_lineage_reference.insert(
                        indexed_location,
                        (
                            lineage.global_reference().clone(),
                            lineages_at_location.len(),
                        ),
                    );
                    lineages_at_location.push(local_reference);

                    lineages_store.push(lineage);
                }

                if sampled_habitat_at_location_max > f64::from(sampled_habitat_at_location) {
                    // Remember the IndexedLocation for another Lineage
                    extra_indexed_locations.push((
                        sampled_habitat_at_location_max - f64::from(sampled_habitat_at_location),
                        IndexedLocation::new(location.clone(), sampled_habitat_at_location),
                    ));
                }
            }
        }

        extra_indexed_locations.sort_by(|(rem_a, _), (rem_b, _)| rem_a.total_cmp(rem_b));

        // Fill up the remaining Lineages prioritiesed by how 'off' the original
        //  allocation was
        while let Some((_, indexed_location)) = extra_indexed_locations.pop() {
            if lineages_store.len() >= total_number_of_lineages {
                break;
            }

            let lineages_at_location = &mut location_to_lineage_references[(
                (indexed_location.location().y() - y_from) as usize,
                (indexed_location.location().x() - x_from) as usize,
            )];

            let lineage = Lineage::new(indexed_location.clone(), habitat);
            let local_reference = InMemoryLineageReference::from(lineages_store.len());

            indexed_location_to_lineage_reference.insert(
                indexed_location,
                (
                    lineage.global_reference().clone(),
                    lineages_at_location.len(),
                ),
            );
            lineages_at_location.push(local_reference);

            lineages_store.push(lineage);
        }

        lineages_store.shrink_to_fit();

        Self {
            lineages_store,
            location_to_lineage_references,
            indexed_location_to_lineage_reference,
            _marker: PhantomData::<H>,
        }
    }
}
