use necsim_core::{
    cogs::{
        CoherentLineageStore, Habitat, LineageReference, RngCore, SeparableDispersalSampler,
        SpeciationProbability,
    },
    landscape::Location,
    simulation::partial::event_sampler::PartialSimulation,
};

use crate::cogs::coalescence_sampler::conditional::ConditionalCoalescenceSampler;

#[allow(clippy::module_name_repetitions)]
pub struct ProbabilityAtLocation {
    speciation: f64,
    out_dispersal: f64,
    self_coalescence: f64,
}

impl ProbabilityAtLocation {
    #[allow(clippy::type_complexity)]
    pub fn new<
        H: Habitat,
        G: RngCore,
        N: SpeciationProbability<H>,
        D: SeparableDispersalSampler<H, G>,
        R: LineageReference<H>,
        S: CoherentLineageStore<H, R>,
    >(
        location: &Location,
        simulation: &PartialSimulation<H, G, N, D, R, S, ConditionalCoalescenceSampler<H, G, R, S>>,
        lineage_store_includes_self: bool,
    ) -> Self {
        let speciation_probability = simulation
            .speciation_probability
            .get_speciation_probability_at_location(location);
        let self_dispersal_probability = simulation
            .dispersal_sampler
            .get_self_dispersal_probability_at_location(location);
        let coalescence_probability_at_location =
            ConditionalCoalescenceSampler::<H, G, R, S>::get_coalescence_probability_at_location(
                location,
                &simulation.habitat,
                &simulation.lineage_store,
                lineage_store_includes_self,
            );

        Self {
            speciation: speciation_probability,
            out_dispersal: (1.0_f64 - speciation_probability)
                * (1.0_f64 - self_dispersal_probability),
            self_coalescence: (1.0_f64 - speciation_probability)
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
