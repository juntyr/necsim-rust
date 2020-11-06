use core::marker::PhantomData;
use core::ops::Index;

use alloc::boxed::Box;
use alloc::vec::Vec;

use array2d::Array2D;

use necsim_core::cogs::Habitat;
use necsim_core::intrinsics::floor;
use necsim_core::landscape::{LandscapeExtent, Location};
use necsim_core::lineage::Lineage;

use crate::cogs::lineage_reference::in_memory::InMemoryLineageReference;

mod store;

#[allow(clippy::module_name_repetitions)]
#[cfg_attr(feature = "cuda", derive(RustToCuda))]
#[cfg_attr(feature = "cuda", r2cBound(H: rust_cuda::common::RustToCuda))]
pub struct IncoherentInMemoryLineageStore<H: Habitat> {
    landscape_extent: LandscapeExtent,
    #[cfg_attr(feature = "cuda", r2cEmbed)]
    lineages_store: Box<[Lineage]>,
    marker: PhantomData<H>,
}

impl<H: Habitat> Index<InMemoryLineageReference> for IncoherentInMemoryLineageStore<H> {
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

impl<H: Habitat> IncoherentInMemoryLineageStore<H> {
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
            0_usize,
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
                    let index_at_location = *lineages_at_location;

                    *lineages_at_location += 1;
                    lineages_store.push(Lineage::new(location.clone(), index_at_location));
                }
            }
        }

        lineages_store.shrink_to_fit();

        Self {
            landscape_extent,
            lineages_store: lineages_store.into_boxed_slice(),
            marker: PhantomData::<H>,
        }
    }
}

impl<H: Habitat> core::fmt::Debug for IncoherentInMemoryLineageStore<H> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("IncoherentInMemoryLineageStore")
            .field("landscape_extent", &self.landscape_extent)
            .field(
                "lineages_store",
                &format_args!(
                    "Box [ {:p}; {} ]",
                    &self.lineages_store,
                    self.lineages_store.len()
                ),
            )
            .field("marker", &self.marker)
            .finish()
    }
}
