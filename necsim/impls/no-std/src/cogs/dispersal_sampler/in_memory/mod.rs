use necsim_core::cogs::{DispersalSampler, F64Core, Habitat, RngCore};

use crate::array2d::Array2D;

pub mod contract;

pub mod alias;
pub mod cumulative;
pub mod packed_alias;
pub mod separable_alias;

use contract::explicit_in_memory_dispersal_check_contract;

#[allow(clippy::module_name_repetitions)]
#[allow(clippy::inline_always, clippy::inline_fn_without_body)]
#[contract_trait]
pub trait InMemoryDispersalSampler<F: F64Core, H: Habitat<F>, G: RngCore<F>>:
    DispersalSampler<F, H, G> + Sized
{
    #[debug_requires((
        dispersal.num_columns() == (
            (habitat.get_extent().width() * habitat.get_extent().height()) as usize
        ) && dispersal.num_rows() == (
            (habitat.get_extent().width() * habitat.get_extent().height()) as usize
        )
    ), "dispersal dimensions are consistent")]
    #[debug_requires(
        explicit_in_memory_dispersal_check_contract(dispersal, habitat),
        "dispersal probabilities are consistent"
    )]
    fn unchecked_new(dispersal: &Array2D<f64>, habitat: &H) -> Self;
}
