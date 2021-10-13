use necsim_core::{
    cogs::{F64Core, Habitat},
    landscape::Location,
};

use crate::array2d::Array2D;

#[allow(clippy::module_name_repetitions)]
pub fn explicit_in_memory_dispersal_check_contract<F: F64Core, H: Habitat<F>>(
    dispersal: &Array2D<f64>,
    habitat: &H,
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
                    }

                    any_dispersal = true;
                } else if dispersal[(row_index, col_index)] < 0.0_f64 {
                    // Dispersal probabilities must be non-negative
                    return false;
                }
            }

            if !any_dispersal {
                // No dispersal from habitat
                return false;
            }
        } else {
            for col_index in 0..dispersal.num_columns() {
                if dispersal[(row_index, col_index)] != 0.0_f64 {
                    // Dispersal probability from non-habitat must be 0.0
                    // - Dispersal from non-habitat (> 0.0)
                    // - Dispersal probabilities must be non-negative (< 0.0)
                    return false;
                }
            }
        }
    }

    true
}
