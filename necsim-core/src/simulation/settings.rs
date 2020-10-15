use crate::landscape::Landscape;

#[allow(clippy::module_name_repetitions)]
pub struct SimulationSettings<L: Landscape> {
    speciation_probability_per_generation: f64,
    sample_percentage: f64,
    landscape: L,
}

impl<L: Landscape> SimulationSettings<L> {
    #[must_use]
    pub fn new(
        speciation_probability_per_generation: f64,
        sample_percentage: f64,
        landscape: L,
    ) -> Self {
        Self {
            speciation_probability_per_generation,
            sample_percentage,
            landscape,
        }
    }

    #[must_use]
    pub fn speciation_probability_per_generation(&self) -> f64 {
        self.speciation_probability_per_generation
    }

    #[must_use]
    pub fn sample_percentage(&self) -> f64 {
        self.sample_percentage
    }

    #[must_use]
    pub fn landscape(&self) -> &L {
        &self.landscape
    }
}
