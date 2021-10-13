use necsim_core::{
    cogs::{
        F64Core, GloballyCoherentLineageStore, Habitat, LineageReference, RngCore,
        SeparableDispersalSampler, SpeciationProbability, TurnoverRate,
    },
    landscape::Location,
};
use necsim_core_bond::ClosedUnitF64;

use crate::cogs::{
    coalescence_sampler::conditional::ConditionalCoalescenceSampler,
    event_sampler::gillespie::GillespiePartialSimulation,
};

#[allow(clippy::module_name_repetitions)]
pub struct ProbabilityAtLocation {
    speciation: ClosedUnitF64,
    out_dispersal: ClosedUnitF64,
    self_coalescence: ClosedUnitF64,
}

impl ProbabilityAtLocation {
    #[allow(clippy::type_complexity)]
    pub fn new<
        F: F64Core,
        H: Habitat<F>,
        G: RngCore<F>,
        R: LineageReference<F, H>,
        S: GloballyCoherentLineageStore<F, H, R>,
        D: SeparableDispersalSampler<F, H, G>,
        T: TurnoverRate<F, H>,
        N: SpeciationProbability<F, H>,
    >(
        location: &Location,
        simulation: &GillespiePartialSimulation<
            F,
            H,
            G,
            R,
            S,
            D,
            ConditionalCoalescenceSampler<F, H, R, S>,
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
            out_dispersal: speciation_probability.one_minus()
                * self_dispersal_probability.one_minus(),
            self_coalescence: speciation_probability.one_minus()
                * self_dispersal_probability
                * coalescence_probability_at_location,
        }
    }

    pub fn speciation(&self) -> ClosedUnitF64 {
        self.speciation
    }

    pub fn out_dispersal(&self) -> ClosedUnitF64 {
        self.out_dispersal
    }

    pub fn self_coalescence(&self) -> ClosedUnitF64 {
        self.self_coalescence
    }

    pub fn total(&self) -> ClosedUnitF64 {
        let total =
            self.speciation().get() + self.out_dispersal().get() + self.self_coalescence().get();

        // Safety: Sum of disjoint event probabilities is in [0.0; 1.0]
        unsafe { ClosedUnitF64::new_unchecked(total) }
    }
}
