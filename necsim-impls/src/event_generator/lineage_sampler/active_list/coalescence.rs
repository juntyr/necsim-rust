use necsim_core::landscape::Location;
use necsim_core::rng::Rng;

use super::{ActiveLineageListSampler, LineageReference};

use crate::event_generator::coalescence_sampler::{
    CoalescenceSampler, ConditionalCoalescenceSampler,
};

#[contract_trait]
impl CoalescenceSampler<LineageReference> for ActiveLineageListSampler {
    #[must_use]
    fn sample_optional_coalescence_at_location(
        &self,
        location: &Location,
        habitat: u32,
        rng: &mut impl Rng,
    ) -> Option<LineageReference> {
        self.lineages_store
            .sample_optional_coalescence_at_location(location, habitat, rng)
    }
}

#[contract_trait]
impl ConditionalCoalescenceSampler<LineageReference> for ActiveLineageListSampler {
    #[must_use]
    fn sample_coalescence_at_location(
        &self,
        location: &Location,
        rng: &mut impl Rng,
    ) -> LineageReference {
        self.lineages_store
            .sample_coalescence_at_location(location, rng)
    }

    #[must_use]
    fn get_coalescence_probability_at_location(&self, location: &Location, habitat: u32) -> f64 {
        self.lineages_store
            .get_coalescence_probability_at_location(location, habitat)
    }
}
