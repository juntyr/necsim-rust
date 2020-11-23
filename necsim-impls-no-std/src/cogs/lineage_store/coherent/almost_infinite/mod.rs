use core::ops::Index;

use alloc::vec::Vec;

use hashbrown::hash_map::HashMap;

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
pub struct CoherentAlmostInfiniteLineageStore {
    landscape_extent: LandscapeExtent,
    lineages_store: Vec<Lineage>,
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
    #[allow(clippy::float_cmp)]
    #[debug_requires(radius < (u32::MAX / 2), "sample circle fits into almost infinite habitat")]
    #[debug_ensures(if sample_percentage == 0.0_f64 {
        ret.get_number_total_lineages() == 0
    } else if sample_percentage == 1.0_f64 {
        ret.get_number_total_lineages() as u64 == habitat.get_total_habitat()
    } else {
        true
    }, "samples active lineages according to settings.sample_percentage()")]
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

        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let skips = (total_area as usize) / sampled_area;

        let mut lineages_store = Vec::with_capacity(sampled_area);
        let mut location_to_lineage_references = HashMap::with_capacity(sampled_area);

        let mut location_id = 0_usize;

        for y in (centre - radius)..=(centre + radius) {
            for x in (centre - radius)..=(centre + radius) {
                let dx = i64::from(x) - i64::from(radius);
                let dy = i64::from(y) - i64::from(radius);

                #[allow(clippy::cast_sign_loss)]
                let distance_squared = (dx * dx) as u64 + (dy * dy) as u64;

                if distance_squared <= radius_squared {
                    if location_id % skips == 0 {
                        let location = Location::new(x, y);

                        lineages_store
                            .push(Lineage::new(IndexedLocation::new(location.clone(), 0_u32)));

                        location_to_lineage_references.insert(
                            location,
                            InMemoryLineageReference::from(lineages_store.len() - 1),
                        );
                    }

                    location_id += 1;
                }
            }
        }

        lineages_store.shrink_to_fit();

        Self {
            landscape_extent: habitat.get_extent(),
            lineages_store,
            location_to_lineage_references,
        }
    }
}
