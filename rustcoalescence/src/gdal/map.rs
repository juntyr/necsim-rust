use std::path::PathBuf;

use anyhow::{Context, Result};
use array2d::Array2D;
use gdal::raster::{Buffer, Dataset};

use super::NewGdalError;

pub fn load_map_f64_from_gdal_raster(path: &PathBuf) -> Result<Array2D<f64>> {
    let dataset = Dataset::open(path)
        .map_err(NewGdalError::from)
        .with_context(|| format!("The map file {:?} could not be opened.", path))?;

    let rasterband = dataset
        .rasterband(1)
        .map_err(NewGdalError::from)
        .with_context(|| {
            format!(
                "The map file {:?} does not contain a raster band at index 1.",
                path
            )
        })?;

    let data: Buffer<f64> = rasterband
        .read_band_as()
        .map_err(NewGdalError::from)
        .with_context(|| format!("The map file {:?} could not be read as Buffer<f64>.", path))?;

    let mut data: Array2D<f64> = Array2D::from_row_major(&data.data, data.size.1, data.size.0);

    let no_data_value = rasterband.no_data_value().unwrap_or(0.0_f64);

    for row_idx in 0..data.num_rows() {
        for col_idx in 0..data.num_columns() {
            #[allow(clippy::float_cmp)]
            if data[(row_idx, col_idx)] == no_data_value {
                data[(row_idx, col_idx)] = 0.0_f64;
            }
        }
    }

    Ok(data)
}
