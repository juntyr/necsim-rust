use core::marker::PhantomData;
use core::ops::Index;

use alloc::vec::Vec;

use array2d::Array2D;

use necsim_core::cogs::Habitat;
use necsim_core::intrinsics::floor;
use necsim_core::landscape::{LandscapeExtent, Location};
use necsim_core::lineage::Lineage;

use crate::cogs::lineage_reference::in_memory::InMemoryLineageReference;

mod store;

#[allow(clippy::module_name_repetitions)]
pub struct CoherentInMemoryLineageStore<H: Habitat> {
    landscape_extent: LandscapeExtent,
    lineages_store: Vec<Lineage>,
    location_to_lineage_references: Array2D<Vec<InMemoryLineageReference>>,
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
    #[debug_ensures(
        ret.landscape_extent == habitat.get_extent(),
        "stores landscape_extent"
    )]
    fn new_impl(sample_percentage: f64, habitat: &H) -> Self {
        #[allow(clippy::cast_possible_truncation)]
        #[allow(clippy::cast_sign_loss)]
        #[allow(clippy::cast_precision_loss)]
        let mut lineages_store =
            Vec::with_capacity(((habitat.get_total_habitat() as f64) * sample_percentage) as usize);

        let landscape_extent = habitat.get_extent();

        let mut location_to_lineage_references = Array2D::filled_with(
            Vec::new(),
            landscape_extent.height() as usize,
            landscape_extent.width() as usize,
        );

        let x_from = landscape_extent.x();
        let y_from = landscape_extent.y();

        for y_offset in 0..landscape_extent.height() {
            for x_offset in 0..landscape_extent.width() {
                let location = Location::new(x_from + x_offset, y_from + y_offset);

                let lineages_at_location =
                    &mut location_to_lineage_references[(y_offset as usize, x_offset as usize)];

                #[allow(clippy::cast_possible_truncation)]
                #[allow(clippy::cast_sign_loss)]
                let sampled_habitat_at_location = floor(
                    f64::from(habitat.get_habitat_at_location(&location)) * sample_percentage,
                ) as usize;

                for _ in 0..sampled_habitat_at_location {
                    let lineage_reference = InMemoryLineageReference::from(lineages_store.len());
                    let index_at_location = lineages_at_location.len();

                    lineages_at_location.push(lineage_reference);
                    lineages_store.push(Lineage::new(location.clone(), index_at_location));
                }
            }
        }

        lineages_store.shrink_to_fit();

        Self {
            landscape_extent,
            lineages_store,
            location_to_lineage_references,
            _marker: PhantomData::<H>,
        }
    }
}
