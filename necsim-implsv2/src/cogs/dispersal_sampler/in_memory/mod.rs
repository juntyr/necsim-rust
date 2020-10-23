use array2d::Array2D;

use necsim_corev2::cogs::{DispersalSampler, Habitat};

pub mod contract;
pub mod error;

pub mod alias;
pub mod cumulative;
pub mod separable_alias;

use contract::explicit_in_memory_dispersal_check_contract;
use error::InMemoryDispersalSamplerError;

#[allow(clippy::module_name_repetitions)]
#[allow(clippy::inline_always, clippy::inline_fn_without_body)]
#[contract_trait]
pub trait InMemoryDispersalSampler<H: Habitat>: DispersalSampler<H> + Sized {
    #[debug_ensures(
        matches!(ret, Err(InMemoryDispersalSamplerError::InconsistentDispersalMapSize)) != (
            dispersal.num_columns() == old(
                (habitat.get_extent().width() * habitat.get_extent().height()) as usize
            ) && dispersal.num_rows() == old(
                (habitat.get_extent().width() * habitat.get_extent().height()) as usize
            )
        ),
        "returns Err(InconsistentDispersalMapSize) iff dispersal dimensions inconsistent"
    )]
    #[debug_ensures(
        matches!(ret, Err(
            InMemoryDispersalSamplerError::InconsistentDispersalProbabilities
        )) != old(
            explicit_in_memory_dispersal_check_contract(dispersal, habitat)
        ), "returns Err(InconsistentDispersalMapSize) iff dispersal dimensions inconsistent"
    )]
    fn new(dispersal: &Array2D<f64>, habitat: &H) -> Result<Self, InMemoryDispersalSamplerError>;
}
