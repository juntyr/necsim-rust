use std::{fs::File, path::PathBuf};

use array2d::Array2D;
use tiff::{decoder::Decoder, tags::Tag};

use anyhow::{Context, Result};

#[path = "data_type.rs"]
mod private;

#[allow(clippy::module_name_repetitions)]
pub trait TiffDataType: private::TiffDataType {}
impl<T: private::TiffDataType> TiffDataType for T {}

#[allow(clippy::module_name_repetitions)]
/// Loads a 2D map from TIFF file at `path` with the data type `D`.
///
/// This function assumes that normal, non-sparse TIFF files are read.
///
/// If `strict_load` is `false`, it checks for the GDAL no data tag and
/// replaces no data values with `D::default()`.
///
/// Furthermore, only the first image is read and any subsequent ones are
/// currently ignored.
pub fn load_map_from_tiff<D: TiffDataType>(
    path: &PathBuf,
    strict_load: bool,
) -> Result<Array2D<D>> {
    let file = File::open(path).context("Could not read file.")?;

    let mut decoder = Decoder::new(file).context("Could not decode TIFF file.")?;

    let colortype = decoder
        .colortype()
        .context("Could not read image colour type.")?;

    if let tiff::ColorType::Gray(bitwidth) = colortype {
        anyhow::ensure!(
            bitwidth == D::BIT_WIDTH,
            format!(
                "Image data format {:?} does not use the correct bitwidth for {}.",
                colortype,
                std::any::type_name::<D>()
            )
        )
    } else {
        anyhow::bail!(format!(
            "Image data format {:?} does not use the correct data format for {}.",
            colortype,
            std::any::type_name::<D>()
        ))
    }

    if let Some(val) = decoder
        .find_tag_unsigned(Tag::SamplesPerPixel)
        .context("Could not read SamplesPerPixel tag.")?
    {
        let samples_per_pixel: u8 = val;

        anyhow::ensure!(
            samples_per_pixel == 1_u8,
            format!(
                "Image must only have one sample per pixel but has {}.",
                samples_per_pixel
            )
        );
    }

    if let Some(vals) = decoder.find_tag_unsigned_vec(Tag::SampleFormat)? {
        let sample_format: Vec<tiff::tags::SampleFormat> = vals
            .into_iter()
            .map(tiff::tags::SampleFormat::from_u16_exhaustive)
            .collect();

        anyhow::ensure!(
            sample_format == vec![D::SAMPLE_FORMAT],
            format!(
                "Image data must have the appropriate sample format {:?} but has {:?}.",
                vec![D::SAMPLE_FORMAT],
                sample_format
            )
        );
    }

    let (width, height) = decoder
        .dimensions()
        .context("Could not read image dimensions.")?;

    let mut image_data = vec![D::default(); (width as usize) * (height as usize)];

    let rows_per_strip = decoder.get_tag_u32(Tag::RowsPerStrip).unwrap_or(height) as usize;
    let samples_per_strip = (width as usize) * rows_per_strip /* only one sample per pixel */;

    for i in 0..(decoder
        .strip_count()
        .context("Could not read strip count.")? as usize)
    {
        let image_buffer = D::decoding_buffer_from_data(&mut image_data[(samples_per_strip * i)..]);

        decoder
            .read_strip_to_buffer(image_buffer)
            .with_context(|| format!("Could not read strip {}.", i))?;
    }

    if !strict_load {
        // If strict loading is disabled, check for a GDAL no data value
        //  and replace it with D::default() (i.e. 0)
        if let Ok(no_data_string) = decoder.get_tag_ascii_string(Tag::GdalNodata) {
            let no_data_value = no_data_string.parse::<D>().with_context(|| {
                format!(
                    "Could not interpret GDAL nodata value {:?} as {}.",
                    no_data_string,
                    std::any::type_name::<D>()
                )
            })?;

            println!("INFO: Used GDAL no data value {:?} ...", no_data_value);

            for elem in &mut image_data {
                if *elem == no_data_value {
                    *elem = D::default();
                }
            }
        } else {
            println!("INFO: No GDAL no data value was found ...");
        }
    }

    Ok(Array2D::from_row_major(&image_data, height as usize, width as usize).unwrap())
}
