use std::path::PathBuf;

use anyhow::{Context, Result};
use array2d::Array2D;
use gdal::raster::{types::GdalType, Buffer, Dataset};

use super::NewGdalError;

pub fn load_map_from_gdal_raster<T: Copy + GdalType>(path: &PathBuf) -> Result<Array2D<T>> {
    let dataset = Dataset::open(path)
        .map_err(NewGdalError::from)
        .with_context(|| format!("The map file {:?} could not be opened.", path))?;

    let data: Buffer<T> = dataset
        .read_full_raster_as(1)
        .map_err(NewGdalError::from)
        .with_context(|| format!("The map file {:?} could not be read.", path))?;

    Ok(Array2D::from_row_major(
        &data.data,
        data.size.1,
        data.size.0,
    ))
}
