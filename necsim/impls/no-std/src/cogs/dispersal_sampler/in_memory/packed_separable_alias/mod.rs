use alloc::{boxed::Box, vec::Vec};
use core::{marker::PhantomData, ops::Range};
use necsim_core_bond::NonNegativeF64;

use r#final::Final;

use necsim_core::{
    cogs::{Backup, Habitat, MathsCore, RngCore},
    landscape::Location,
};

use crate::{alias::packed::AliasMethodSamplerAtom, array2d::Array2D};

mod dispersal;

use super::InMemoryDispersalSampler;

#[derive(Clone, Debug, TypeLayout)]
#[allow(clippy::module_name_repetitions)]
#[doc(hidden)]
#[repr(C)]
pub struct SeparableAliasSamplerRange {
    start: usize,
    ledge: usize,
    end: usize,
}

impl From<SeparableAliasSamplerRange> for Range<usize> {
    fn from(range: SeparableAliasSamplerRange) -> Self {
        range.start..range.end
    }
}

#[allow(clippy::module_name_repetitions)]
#[cfg_attr(feature = "cuda", derive(rust_cuda::lend::LendRustToCuda))]
#[cfg_attr(feature = "cuda", cuda(free = "M", free = "H", free = "G"))]
pub struct InMemoryPackedSeparableAliasDispersalSampler<M: MathsCore, H: Habitat<M>, G: RngCore<M>>
{
    #[cfg_attr(feature = "cuda", cuda(embed))]
    alias_dispersal_ranges: Final<Array2D<SeparableAliasSamplerRange>>,
    #[cfg_attr(feature = "cuda", cuda(embed))]
    alias_dispersal_buffer: Final<Box<[AliasMethodSamplerAtom<usize>]>>,
    marker: PhantomData<(M, H, G)>,
}

#[contract_trait]
impl<M: MathsCore, H: Habitat<M>, G: RngCore<M>> InMemoryDispersalSampler<M, H, G>
    for InMemoryPackedSeparableAliasDispersalSampler<M, H, G>
{
    /// Creates a new `InMemoryPackedSeparableAliasDispersalSampler` from the
    /// `dispersal` map and extent of the habitat map.
    fn unchecked_new(dispersal: &Array2D<NonNegativeF64>, habitat: &H) -> Self {
        let habitat_extent = habitat.get_extent();

        let mut event_weights: Vec<(usize, NonNegativeF64)> =
            Vec::with_capacity(dispersal.row_len());

        let mut alias_dispersal_buffer = Vec::new();

        let alias_dispersal_ranges = Array2D::from_iter_row_major(
            dispersal.rows_iter().enumerate().map(|(row_index, row)| {
                event_weights.clear();

                for (col_index, dispersal_probability) in row.enumerate() {
                    #[allow(clippy::cast_possible_truncation)]
                    let location =
                        Location::new(
                            habitat_extent.origin().x().wrapping_add(
                                (col_index % usize::from(habitat_extent.width())) as u32,
                            ),
                            habitat_extent.origin().y().wrapping_add(
                                (col_index / usize::from(habitat_extent.width())) as u32,
                            ),
                        );

                    // Multiply all dispersal probabilities by the habitat of their target
                    let weight = *dispersal_probability
                        * NonNegativeF64::from(habitat.get_habitat_at_location(&location));

                    if weight > 0.0_f64 {
                        event_weights.push((col_index, weight));
                    }
                }

                let range_from = alias_dispersal_buffer.len();

                if event_weights.is_empty() {
                    SeparableAliasSamplerRange {
                        start: range_from,
                        ledge: range_from,
                        end: range_from,
                    }
                } else {
                    // sort the alias sampling atoms to push self-dispersal to the right
                    let mut atoms = AliasMethodSamplerAtom::create(&event_weights);
                    atoms.sort_by_key(|a| {
                        usize::from(*a.e() == row_index) + usize::from(*a.k() == row_index)
                    });

                    // find the index amongst the atoms of the first atom that includes
                    //  self-dispersal, either with u < 1.0 (uniquely) or u = 1.0 (iff
                    //  no self-dispersal with u < 1.0 exists)
                    let ledge = match atoms.binary_search_by_key(&1, |a| {
                        usize::from(*a.e() == row_index) + usize::from(*a.k() == row_index)
                    }) {
                        Ok(i) | Err(i) => i,
                    };

                    alias_dispersal_buffer.append(&mut atoms);

                    SeparableAliasSamplerRange {
                        start: range_from,
                        ledge: range_from + ledge,
                        end: alias_dispersal_buffer.len(),
                    }
                }
            }),
            usize::from(habitat_extent.height()),
            usize::from(habitat_extent.width()),
        )
        .unwrap(); // infallible by PRE;

        Self {
            alias_dispersal_ranges: Final::new(alias_dispersal_ranges),
            alias_dispersal_buffer: Final::new(alias_dispersal_buffer.into_boxed_slice()),
            marker: PhantomData::<(M, H, G)>,
        }
    }
}

impl<M: MathsCore, H: Habitat<M>, G: RngCore<M>> core::fmt::Debug
    for InMemoryPackedSeparableAliasDispersalSampler<M, H, G>
{
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        f.debug_struct(stringify!(InMemoryPackedSeparableAliasDispersalSampler))
            .field("alias_dispersal_ranges", &self.alias_dispersal_ranges)
            .field(
                "alias_dispersal_buffer",
                &format_args!(
                    "Box [ {:p}; {} ]",
                    &self.alias_dispersal_buffer,
                    self.alias_dispersal_buffer.len()
                ),
            )
            .finish()
    }
}

#[contract_trait]
impl<M: MathsCore, H: Habitat<M>, G: RngCore<M>> Backup
    for InMemoryPackedSeparableAliasDispersalSampler<M, H, G>
{
    unsafe fn backup_unchecked(&self) -> Self {
        Self {
            alias_dispersal_ranges: Final::new(self.alias_dispersal_ranges.clone()),
            alias_dispersal_buffer: Final::new(self.alias_dispersal_buffer.clone()),
            marker: PhantomData::<(M, H, G)>,
        }
    }
}
