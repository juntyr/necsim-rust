use necsim_core::cogs::{DispersalSampler, Habitat, MathsCore, RngCore};

use crate::array2d::Array2D;

pub mod contract;

pub mod alias;
pub mod cumulative;
pub mod packed_alias;
pub mod separable_alias;

use contract::explicit_in_memory_dispersal_check_contract;
use necsim_core_bond::NonNegativeF64;

#[allow(clippy::module_name_repetitions)]
#[allow(clippy::inline_always, clippy::inline_fn_without_body)]
#[contract_trait]
pub trait InMemoryDispersalSampler<M: MathsCore, H: Habitat<M>, G: RngCore<M>>:
    DispersalSampler<M, H, G> + Sized
{
    #[debug_requires((
        dispersal.num_columns() == (
            usize::from(habitat.get_extent().width()) * usize::from(habitat.get_extent().height())
        ) && dispersal.num_rows() == (
            usize::from(habitat.get_extent().width()) * usize::from(habitat.get_extent().height())
        )
    ), "dispersal dimensions are consistent")]
    #[debug_requires(
        explicit_in_memory_dispersal_check_contract(dispersal, habitat),
        "dispersal probabilities are consistent"
    )]
    fn unchecked_new(dispersal: &Array2D<NonNegativeF64>, habitat: &H) -> Self;
}
