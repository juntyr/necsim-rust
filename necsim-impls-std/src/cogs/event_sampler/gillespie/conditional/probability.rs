use necsim_core::cogs::{Habitat, LineageReference, LineageStore};
use necsim_core::landscape::Location;
use necsim_core::simulation::partial::event_sampler::PartialSimulation;

use necsim_impls_no_std::cogs::coalescence_sampler::conditional::ConditionalCoalescenceSampler;

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
        simulation: &PartialSimulation<H, D, R, S, ConditionalCoalescenceSampler<H, R, S>>,
        lineage_store_includes_self: bool,
    ) -> Self {
        let self_dispersal_probability = simulation
            .dispersal_sampler
            .get_self_dispersal_probability_at_location(location);
        let coalescence_probability_at_location =
            ConditionalCoalescenceSampler::get_coalescence_probability_at_location(
                location,
                simulation.habitat,
                simulation.lineage_store,
                lineage_store_includes_self,
            );

        Self {
            speciation: *simulation.speciation_probability_per_generation,
            out_dispersal: (1.0_f64 - simulation.speciation_probability_per_generation)
                * (1.0_f64 - self_dispersal_probability),
            self_coalescence: (1.0_f64 - simulation.speciation_probability_per_generation)
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
