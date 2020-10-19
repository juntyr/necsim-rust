use array2d::Array2D;

use crate::landscape::habitat::Habitat;
use necsim_core::landscape::Location;

use super::InMemoryPrecalculatedDispersal;

#[allow(clippy::module_name_repetitions)]
pub fn explicit_in_memory_precalculated_dispersal_check_contract(
    dispersal: &Array2D<f64>,
    habitat: &impl Habitat,
) -> bool {
    let habitat_width = habitat.get_extent().width();

    for row_index in 0..dispersal.num_rows() {
        #[allow(clippy::cast_possible_truncation)]
        let dispersal_origin = Location::new(
            (row_index % habitat_width as usize) as u32,
            (row_index / habitat_width as usize) as u32,
        );

        if habitat.get_habitat_at_location(&dispersal_origin) > 0 {
            let mut any_dispersal = false;

            for col_index in 0..dispersal.num_columns() {
                #[allow(clippy::cast_possible_truncation)]
                let dispersal_target = Location::new(
                    (col_index % habitat_width as usize) as u32,
                    (col_index / habitat_width as usize) as u32,
                );

                if dispersal[(row_index, col_index)] > 0.0_f64 {
                    if habitat.get_habitat_at_location(&dispersal_target) == 0 {
                        // Dispersal from habitat to non-habitat
                        return false;
                    } else {
                        any_dispersal = true;
                    }
                }
            }

            if !any_dispersal {
                // No dispersal from habitat
                return false;
            }
        } else {
            for col_index in 0..dispersal.num_columns() {
                if dispersal[(row_index, col_index)] > 0.0_f64 {
                    // Dispersal from non-habitat
                    return false;
                }
            }
        }
    }

    true
}

impl InMemoryPrecalculatedDispersal {
    pub(super) fn explicit_only_valid_targets_dispersal_contract(
        &self,
        habitat: &impl Habitat,
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
