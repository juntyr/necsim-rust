use necsim_core::landscape::Location;
use necsim_core::rng::Rng;

use super::{GillespieLineageSampler, LineageReference};

use crate::event_generator::coalescence_sampler::CoalescenceSampler;

#[contract_trait]
impl CoalescenceSampler<LineageReference> for GillespieLineageSampler {
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
