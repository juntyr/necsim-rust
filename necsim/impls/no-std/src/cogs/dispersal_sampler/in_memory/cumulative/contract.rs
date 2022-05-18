use necsim_core::{
    cogs::{rng::UniformClosedOpenUnit, DistributionSampler, Habitat, MathsCore, Rng},
    landscape::Location,
};

use super::InMemoryCumulativeDispersalSampler;

impl<M: MathsCore, H: Habitat<M>, G: Rng<M>> InMemoryCumulativeDispersalSampler<M, H, G>
where
    G::Sampler: DistributionSampler<M, G::Generator, G::Sampler, UniformClosedOpenUnit>,
{
    pub(super) fn explicit_only_valid_targets_dispersal_contract(&self, habitat: &H) -> bool {
        let habitat_width = habitat.get_extent().width();

        for target_index in self.valid_dispersal_targets.iter().filter_map(|x| *x) {
            #[allow(clippy::cast_possible_truncation)]
            let dispersal_target = Location::new(
                (target_index % usize::from(habitat_width)) as u32,
                (target_index / usize::from(habitat_width)) as u32,
            );

            if habitat.get_habitat_at_location(&dispersal_target) == 0 {
                // Possible dispersal to non-habitat
                return false;
            }
        }

        true
    }
}
