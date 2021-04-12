use necsim_core::{
    cogs::{
        GloballyCoherentLineageStore, Habitat, LineageReference, RngCore,
        SeparableDispersalSampler, SpeciationProbability, TurnoverRate,
    },
    landscape::Location,
};

use crate::cogs::{
    coalescence_sampler::conditional::ConditionalCoalescenceSampler,
    event_sampler::gillespie::GillespiePartialSimulation,
};

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
        R: LineageReference<H>,
        S: GloballyCoherentLineageStore<H, R>,
        D: SeparableDispersalSampler<H, G>,
        T: TurnoverRate<H>,
        N: SpeciationProbability<H>,
    >(
        location: &Location,
        simulation: &GillespiePartialSimulation<
            H,
            G,
            R,
            S,
            D,
            ConditionalCoalescenceSampler<H, R, S>,
            T,
            N,
        >,
        lineage_store_includes_self: bool,
    ) -> Self {
        let speciation_probability = simulation
            .speciation_probability
            .get_speciation_probability_at_location(location, &simulation.habitat);
        let self_dispersal_probability = simulation
            .dispersal_sampler
            .get_self_dispersal_probability_at_location(location, &simulation.habitat);
        let coalescence_probability_at_location =
            ConditionalCoalescenceSampler::get_coalescence_probability_at_location(
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
