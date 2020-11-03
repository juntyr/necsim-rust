use std::path::PathBuf;

use anyhow::{Context, Result};
use array2d::Array2D;
use cast::{Error, From as _From};

use super::gdal::load_map_f64_from_gdal_raster;

pub fn load_dispersal_map(path: &PathBuf) -> Result<Array2D<f64>> {
    load_map_f64_from_gdal_raster(path).context("Failed to load the dispersal map")
}

pub fn load_habitat_map(path: &PathBuf, dispersal: &Array2D<f64>) -> Result<Array2D<u32>> {
    let habitat_f64 =
        load_map_f64_from_gdal_raster(path).context("Failed to load the habitat map")?;

    let mut habitat: Array2D<u32> =
        Array2D::filled_with(0, habitat_f64.num_rows(), habitat_f64.num_columns());

    for y in 0..habitat_f64.num_rows() {
        for x in 0..habitat_f64.num_columns() {
            let h_f64 = habitat_f64[(y, x)];

            habitat[(y, x)] = if h_f64 < 0.0_f64 {
                Err(Error::Underflow)
            } else if h_f64 < 1.0_f64 {
                // If there is any dispersal from this location, it must be habitat
                if dispersal
                    .row_iter(y * habitat.num_columns() + x)
                    .map_or(false, |mut it| it.any(|p| *p > 0.0_f64))
                {
                    Ok(1)
                } else {
                    Ok(0)
                }
            } else {
                u32::cast(h_f64)
            }
            .context("Failed to interpret the habitat map as u32")?;
        }
    }

    Ok(habitat)
}
