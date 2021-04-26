use std::path::Path;

use anyhow::{Context, Result};
use array2d::Array2D;

use crate::args::MapLoadingMode;

pub fn load_dispersal_map(path: &Path, loading_mode: MapLoadingMode) -> Result<Array2D<f64>> {
    crate::tiff::load_map_from_tiff::<f64>(
        path,
        match loading_mode {
            MapLoadingMode::FixMe | MapLoadingMode::OffByOne => false,
            MapLoadingMode::Strict => true,
        },
    )
    .with_context(|| format!("Failed to load the dispersal map from {:?}.", path))
}

pub fn load_habitat_map(
    path: &Path,
    dispersal: &mut Array2D<f64>,
    loading_mode: MapLoadingMode,
) -> Result<Array2D<u32>> {
    let mut habitat = crate::tiff::load_map_from_tiff::<u32>(
        path,
        match loading_mode {
            MapLoadingMode::FixMe | MapLoadingMode::OffByOne => false,
            MapLoadingMode::Strict => true,
        },
    )
    .with_context(|| format!("Failed to load the habiat map from {:?}.", path))?;

    match loading_mode {
        MapLoadingMode::FixMe => {
            fix_habitat_map(&mut habitat, dispersal);
            fix_dispersal_map(&habitat, dispersal);
        },
        MapLoadingMode::OffByOne => fix_habitat_map(&mut habitat, &dispersal),
        MapLoadingMode::Strict => (),
    };

    Ok(habitat)
}

// Fix habitat rounding error by correcting 0/1 values to 0/1 based on dispersal
//  (can only disperse from habitat)
fn fix_habitat_map(habitat: &mut Array2D<u32>, dispersal: &Array2D<f64>) {
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

                    // warn!(
                    //     "Corrected habitat value {} to {} at ({},{}) to fit \
                    //      dispersal ...", h_before, h_fixed, x, y
                    // );
                }
            }
        }
    }
}

// Fix dispersal by removing dispersal to/from non-habitat
// Fix dispersal by adding self-dispersal when no dispersal exists from habitat
fn fix_dispersal_map(habitat: &Array2D<u32>, dispersal: &mut Array2D<f64>) {
    let size = habitat.num_rows() * habitat.num_columns();

    for y in 0..habitat.num_rows() {
        for x in 0..habitat.num_columns() {
            let row = y * habitat.num_columns() + x;

            if habitat[(y, x)] == 0 {
                let mut warn = false;

                // Fix dispersal from non-habitat
                for column in 0..size {
                    let d = &mut dispersal[(row, column)];

                    if *d > 0.0_f64 {
                        warn = true;

                        *d = 0.0_f64;
                    }
                }

                if warn {
                    // warn!(
                    //     "Corrected dispersal values to 0.0 at ({},{}) to fit
                    // \      habitat ...", x, y,
                    // );
                }
            } else {
                let mut total_dispersal = 0.0_f64;

                // Fix dispersal to non-habitat
                for column in 0..size {
                    let tx = column % habitat.num_columns();
                    let ty = column / habitat.num_columns();

                    let d = &mut dispersal[(row, column)];

                    if habitat[(ty, tx)] == 0 && *d > 0.0_f64 {
                        // warn!(
                        //     "Corrected dispersal value {} to 0.0 at ({},{})->({},{}) to fit \
                        //      habitat ...",
                        //     *d, x, y, tx, ty,
                        // );

                        *d = 0.0_f64;
                    }

                    total_dispersal += *d;
                }

                // Fix no dispersal from habitat with self-dispersal
                if total_dispersal <= 0.0_f64 {
                    dispersal[(row, row)] = 1.0_f64;
                }
            }
        }
    }
}
