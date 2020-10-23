use necsim_corev2::cogs::{Habitat, LineageReference, LineageStore};
use necsim_corev2::landscape::Location;

use crate::cogs::coalescence_sampler::conditional::ConditionalCoalescenceSampler;
use crate::cogs::dispersal_sampler::separable::SeparableDispersalSampler;

#[allow(clippy::module_name_repetitions)]
pub struct ProbabilityAtLocation {
    speciation: f64,
    out_dispersal: f64,
    self_coalescence: f64,
}

impl ProbabilityAtLocation {
    pub fn new<
        H: Habitat,
        D: SeparableDispersalSampler<H>,
        R: LineageReference<H>,
        S: LineageStore<H, R>,
    >(
        location: &Location,
        speciation_probability_per_generation: f64,
        habitat: &H,
        dispersal_sampler: &D,
        lineage_store: &S,
        lineage_store_includes_self: bool,
    ) -> Self {
        let self_dispersal_probability =
            dispersal_sampler.get_self_dispersal_probability_at_location(location);
        let coalescence_probability_at_location =
            ConditionalCoalescenceSampler::get_coalescence_probability_at_location(
                location,
                habitat,
                lineage_store,
                lineage_store_includes_self,
            );

        Self {
            speciation: speciation_probability_per_generation,
            out_dispersal: (1.0_f64 - speciation_probability_per_generation)
                * (1.0_f64 - self_dispersal_probability),
            self_coalescence: (1.0_f64 - speciation_probability_per_generation)
                * self_dispersal_probability
                * coalescence_probability_at_location,
        }
    }

    pub fn speciation(&self) -> f64 {
        self.speciation
    }

    pub fn out_dispersal(&self) -> f64 {
        self.out_dispersal
    }

    pub fn self_coalescence(&self) -> f64 {
        self.self_coalescence
    }

    pub fn total(&self) -> f64 {
        self.speciation() + self.out_dispersal() + self.self_coalescence()
    }
}
