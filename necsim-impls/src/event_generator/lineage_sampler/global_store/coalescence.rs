use necsim_core::landscape::Location;
use necsim_core::rng::Rng;

use super::{GlobalLineageStore, LineageReference};

use crate::event_generator::coalescence_sampler::{
    CoalescenceSampler, ConditionalCoalescenceSampler,
};

#[contract_trait]
impl CoalescenceSampler<LineageReference> for GlobalLineageStore {
    #[must_use]
    #[debug_requires(
        self.landscape_extent.contains(location),
        "location is inside landscape extent"
    )]
    #[debug_ensures(
        ret.is_some() -> self.explicit_global_store_lineage_at_location_contract(ret.unwrap()),
        "lineage is at the location and index it references"
    )]
    fn sample_optional_coalescence_at_location(
        &self,
        location: &Location,
        habitat: u32,
        rng: &mut impl Rng,
    ) -> Option<LineageReference> {
        let lineages_at_location = self.get_active_lineages_at_location(location);
        let population = lineages_at_location.len();

        let chosen_coalescence = rng.sample_index(habitat as usize);

        if chosen_coalescence >= population {
            return None;
        }

        Some(lineages_at_location[chosen_coalescence])
    }
}

#[contract_trait]
impl ConditionalCoalescenceSampler<LineageReference> for GlobalLineageStore {
    #[must_use]
    #[debug_requires(
        self.landscape_extent.contains(location),
        "location is inside landscape extent"
    )]
    #[debug_requires(
        !self.get_active_lineages_at_location(location).is_empty(),
        "there are some active lineages at the location"
    )]
    #[debug_ensures(
        self.explicit_global_store_lineage_at_location_contract(ret),
        "lineage is at the location and index it references"
    )]
    fn sample_coalescence_at_location(
        &self,
        location: &Location,
        rng: &mut impl Rng,
    ) -> LineageReference {
        let lineages_at_location = self.get_active_lineages_at_location(location);
        let population = lineages_at_location.len();

        let chosen_coalescence = rng.sample_index(population);

        lineages_at_location[chosen_coalescence]
    }

    #[must_use]
    #[debug_requires(
        self.landscape_extent.contains(location),
        "location is inside landscape extent"
    )]
    fn get_coalescence_probability_at_location(&self, location: &Location, habitat: u32) -> f64 {
        #[allow(clippy::cast_precision_loss)]
        let population_at_location = self.get_active_lineages_at_location(location).len() as f64;

        population_at_location / f64::from(habitat)
    }
}
