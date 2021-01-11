use core::ops::Index;

use alloc::{boxed::Box, vec::Vec};

use necsim_core::{
    cogs::{Habitat, LineageStore},
    intrinsics::floor,
    landscape::{IndexedLocation, LandscapeExtent, Location},
    lineage::Lineage,
};

use crate::cogs::{
    habitat::almost_infinite::AlmostInfiniteHabitat,
    lineage_reference::in_memory::InMemoryLineageReference,
};

mod store;

#[allow(clippy::module_name_repetitions)]
#[derive(Debug)]
#[cfg_attr(feature = "cuda", derive(RustToCuda))]
pub struct IncoherentAlmostInfiniteLineageStore {
    landscape_extent: LandscapeExtent,
    #[cfg_attr(feature = "cuda", r2cEmbed)]
    lineages_store: Box<[Lineage]>,
}

impl Index<InMemoryLineageReference> for IncoherentAlmostInfiniteLineageStore {
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

impl IncoherentAlmostInfiniteLineageStore {
    #[must_use]
    #[allow(clippy::float_cmp)]
    #[debug_requires(radius < (u32::MAX / 2), "sample circle fits into almost infinite habitat")]
    #[debug_ensures(
        sample_percentage == 0.0_f64 -> ret.get_number_total_lineages() == 0,
        "samples active lineages according to sample_percentage()"
    )]
    #[debug_ensures(
        ret.landscape_extent == habitat.get_extent(),
        "stores landscape_extent"
    )]
    pub fn new(radius: u32, sample_percentage: f64, habitat: &AlmostInfiniteHabitat) -> Self {
        let centre = u32::MAX / 2;

        let radius_squared = u64::from(radius) * u64::from(radius);

        #[allow(clippy::cast_possible_truncation, clippy::cast_precision_loss)]
        let total_area = (radius_squared as f64) * core::f64::consts::PI;
        #[allow(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            clippy::cast_precision_loss
        )]
        let sampled_area = floor(total_area * sample_percentage) as usize;

        let mut lineages_store = Vec::with_capacity(sampled_area);

        if sampled_area > 0 {
            let mut location_id = 0_usize;

            for y in (centre - radius)..=(centre + radius) {
                for x in (centre - radius)..=(centre + radius) {
                    let dx = i64::from(x) - i64::from(centre);
                    let dy = i64::from(y) - i64::from(centre);

                    #[allow(clippy::cast_sign_loss)]
                    let distance_squared = (dx * dx) as u64 + (dy * dy) as u64;

                    if distance_squared <= radius_squared {
                        #[allow(clippy::cast_precision_loss)]
                        if floor((location_id as f64) * sample_percentage)
                            < floor(((location_id + 1) as f64) * sample_percentage)
                        {
                            let location = Location::new(x, y);

                            lineages_store.push(Lineage::new(
                                IndexedLocation::new(location.clone(), 0_u32),
                                habitat,
                            ));
                        }

                        location_id += 1;
                    }
                }
            }
        }

        lineages_store.shrink_to_fit();

        Self {
            landscape_extent: habitat.get_extent(),
            lineages_store: lineages_store.into_boxed_slice(),
        }
    }
}
