use std::path::PathBuf;

use anyhow::{Context, Result};
use array2d::Array2D;

pub fn load_dispersal_map(path: &PathBuf, strict_load: bool) -> Result<Array2D<f64>> {
    crate::tiff::load_map_from_tiff::<f64>(path, strict_load)
        .with_context(|| format!("Failed to load the dispersal map from {:?}.", path))
}

pub fn load_habitat_map(
    path: &PathBuf,
    dispersal: &Array2D<f64>,
    strict_load: bool,
) -> Result<Array2D<u32>> {
    let mut habitat = crate::tiff::load_map_from_tiff::<u32>(path, strict_load)
        .with_context(|| format!("Failed to load the habiat map from {:?}.", path))?;

    if !strict_load {
        for y in 0..habitat.num_rows() {
            for x in 0..habitat.num_columns() {
                let h_before = habitat[(y, x)];

                if h_before <= 1 {
                    // If there is any dispersal from this location, it must be habitat
                    let h_fixed = if dispersal
                        .row_iter(y * habitat.num_columns() + x)
                        .map_or(false, |mut it| it.any(|p| *p > 0.0_f64))
                    {
                        1
                    } else {
                        0
                    };

                    if h_fixed != h_before {
                        habitat[(y, x)] = h_fixed;

                        println!(
                            "INFO: Corrected habitat value {} to {} at ({},{}) to fit dispersal \
                             ...",
                            h_before, h_fixed, x, y
                        );
                    }
                }
            }
        }
    }

    Ok(habitat)
}
