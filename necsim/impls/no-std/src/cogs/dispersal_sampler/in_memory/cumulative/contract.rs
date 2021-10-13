use necsim_core::{
    cogs::{Habitat, MathsCore},
    landscape::Location,
};

use super::InMemoryCumulativeDispersalSampler;

impl InMemoryCumulativeDispersalSampler {
    pub(super) fn explicit_only_valid_targets_dispersal_contract<M: MathsCore, H: Habitat<M>>(
        &self,
        habitat: &H,
    ) -> bool {
        let habitat_width = habitat.get_extent().width();

        for target_index in self.valid_dispersal_targets.iter().filter_map(|x| *x) {
            #[allow(clippy::cast_possible_truncation)]
            let dispersal_target = Location::new(
                (target_index % habitat_width as usize) as u32,
                (target_index / habitat_width as usize) as u32,
            );

            if habitat.get_habitat_at_location(&dispersal_target) == 0 {
                // Possible dispersal to non-habitat
                return false;
            }
        }

        true
    }
}
