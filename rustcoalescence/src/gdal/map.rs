//use std::any::type_name;
use std::path::PathBuf;

use anyhow::{Context, Result};
use array2d::Array2D;
use gdal::raster::{types::GdalType, Buffer, Dataset};

use super::NewGdalError;

pub trait GdalTypeConvert: GdalType + Sized {
    fn convert(value: f64) -> Option<Self>;
}

impl GdalTypeConvert for f64 {
    fn convert(value: f64) -> Option<f64> {
        Some(value)
    }
}

impl GdalTypeConvert for u32 {
    fn convert(mut value: f64) -> Option<u32> {
        if value < 0.0_f64 {
            return None;
        }

        // Should we maybe go back and make this application logic
        // i.e. test if dispersal but habitat 0 and only then round up?
        value = value.ceil();

        /*if value.fract() > 0.0_f64 {
            return None;
        }*/

        if value > u32::MAX.into() {
            return None;
        }

        #[allow(clippy::cast_sign_loss)]
        #[allow(clippy::cast_possible_truncation)]
        Some(value as u32)
    }
}

pub fn load_map_from_gdal_raster<T: Copy + Default + GdalTypeConvert>(
    path: &PathBuf,
) -> Result<Array2D<T>> {
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

    /*println!("Band type is {:?}!", rasterband.band_type());
    println!("No data value is {:?}!", rasterband.no_data_value());
    println!("Scale is {:?}!", rasterband.scale());
    println!("Offset is {:?}!", rasterband.offset());*/

    let data: Buffer<f64> = rasterband
        .read_band_as()
        .map_err(NewGdalError::from)
        .with_context(|| format!("The map file {:?} could not be read as Buffer<f64>.", path))?;

    let array_f64: Array2D<f64> = Array2D::from_row_major(&data.data, data.size.1, data.size.0);

    let no_data_value: f64 = rasterband.no_data_value().unwrap_or(0.0_f64);

    let mut array_t: Array2D<T> =
        Array2D::filled_with(T::default(), array_f64.num_rows(), array_f64.num_columns());

    for row_index in 0..array_f64.num_rows() {
        for col_index in 0..array_f64.num_columns() {
            let datum_f64 = array_f64[(row_index, col_index)];

            #[allow(clippy::float_cmp)]
            if datum_f64 != no_data_value {
                // TODO: perform error matching here to get proper reporting
                array_t[(row_index, col_index)] = T::convert(datum_f64).unwrap();
            }
        }
    }

    Ok(array_t)
}
