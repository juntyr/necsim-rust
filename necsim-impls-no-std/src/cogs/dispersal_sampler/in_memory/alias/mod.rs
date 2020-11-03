use alloc::boxed::Box;
use alloc::vec::Vec;
use core::ops::Range;

use array2d::{Array2D, Error};

use necsim_core::cogs::Habitat;
use necsim_core::landscape::{LandscapeExtent, Location};

mod dispersal;

use crate::alias::AliasMethodSamplerAtom;

use super::InMemoryDispersalSampler;

#[derive(Clone)]
#[allow(clippy::module_name_repetitions)]
pub struct AliasSamplerRange(Range<usize>);

impl From<Range<usize>> for AliasSamplerRange {
    fn from(range: Range<usize>) -> Self {
        Self(range)
    }
}

impl Into<Range<usize>> for AliasSamplerRange {
    fn into(self) -> Range<usize> {
        self.0
    }
}

#[cfg(feature = "cuda")]
unsafe impl rustacuda_core::DeviceCopy for AliasSamplerRange {}

#[allow(clippy::module_name_repetitions)]
#[cfg_attr(feature = "cuda", derive(RustToCuda, LendToCuda))]
pub struct InMemoryAliasDispersalSampler {
    #[cfg_attr(feature = "cuda", r2cEmbed)]
    alias_dispersal_ranges: Array2D<AliasSamplerRange>,
    #[cfg_attr(feature = "cuda", r2cEmbed)]
    alias_dispersal_buffer: Box<[AliasMethodSamplerAtom<usize>]>,
    habitat_extent: LandscapeExtent,
}

#[contract_trait]
impl<H: Habitat> InMemoryDispersalSampler<H> for InMemoryAliasDispersalSampler {
    /// Creates a new `InMemoryAliasDispersalSampler` from the
    /// `dispersal` map and extent of the habitat map.
    ///
    /// # Errors
    ///
    /// `Err(_)` is returned iff the dispersal `Array2D` cannot
    /// be constructed successfully.
    fn unchecked_new(dispersal: &Array2D<f64>, habitat: &H) -> Result<Self, Error> {
        let habitat_extent = habitat.get_extent();

        let mut event_weights: Vec<(usize, f64)> = Vec::with_capacity(dispersal.row_len());

        let mut alias_dispersal_buffer = Vec::new();

        let alias_dispersal_ranges = Array2D::from_iter_row_major(
            dispersal.rows_iter().map(|row| {
                event_weights.clear();

                for (col_index, dispersal_probability) in row.enumerate() {
                    #[allow(clippy::cast_possible_truncation)]
                    let location = Location::new(
                        (col_index % (habitat_extent.width() as usize)) as u32 + habitat_extent.x(),
                        (col_index / (habitat_extent.width() as usize)) as u32 + habitat_extent.y(),
                    );

                    // Multiply all dispersal probabilities by the habitat of their target
                    let weight = dispersal_probability
                        * f64::from(habitat.get_habitat_at_location(&location));

                    if weight > 0.0_f64 {
                        event_weights.push((col_index, weight));
                    }
                }

                let range_from = alias_dispersal_buffer.len();

                if event_weights.is_empty() {
                    AliasSamplerRange::from(range_from..range_from)
                } else {
                    alias_dispersal_buffer
                        .append(&mut AliasMethodSamplerAtom::create(&event_weights));

                    AliasSamplerRange::from(range_from..alias_dispersal_buffer.len())
                }
            }),
            habitat_extent.height() as usize,
            habitat_extent.width() as usize,
        )?;

        Ok(Self {
            alias_dispersal_ranges,
            alias_dispersal_buffer: alias_dispersal_buffer.into_boxed_slice(),
            habitat_extent,
        })
    }
}
