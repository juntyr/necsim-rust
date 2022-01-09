use necsim_core::cogs::{Habitat, MathsCore, RngCore};
use necsim_impls_no_std::array2d::Array2D;

pub mod error;

use error::InMemoryDispersalSamplerError;
use necsim_core_bond::NonNegativeF64;
use necsim_impls_no_std::cogs::dispersal_sampler::in_memory::{
    contract::explicit_in_memory_dispersal_check_contract,
    InMemoryDispersalSampler as InMemoryDispersalSamplerNoError,
};

#[allow(clippy::module_name_repetitions)]
#[allow(clippy::inline_always, clippy::inline_fn_without_body)]
#[contract_trait]
pub trait InMemoryDispersalSampler<M: MathsCore, H: Habitat<M>, G: RngCore<M>>:
    InMemoryDispersalSamplerNoError<M, H, G> + Sized
{
    #[debug_ensures(
        matches!(ret, Err(InMemoryDispersalSamplerError::InconsistentDispersalMapSize)) != (
            dispersal.num_columns() == old(
                usize::from(habitat.get_extent().width()) * usize::from(habitat.get_extent().height())
            ) && dispersal.num_rows() == old(
                usize::from(habitat.get_extent().width()) * usize::from(habitat.get_extent().height())
            )
        ),
        "returns Err(InconsistentDispersalMapSize) iff the dispersal dimensions are inconsistent"
    )]
    #[debug_ensures(
        matches!(ret, Err(
            InMemoryDispersalSamplerError::InconsistentDispersalProbabilities
        )) != old(
            explicit_in_memory_dispersal_check_contract(dispersal, habitat)
        ), "returns Err(InconsistentDispersalProbabilities) iff the dispersal probabilities are inconsistent"
    )]
    fn new(
        dispersal: &Array2D<NonNegativeF64>,
        habitat: &H,
    ) -> Result<Self, InMemoryDispersalSamplerError>;
}

#[contract_trait]
impl<M: MathsCore, H: Habitat<M>, G: RngCore<M>, T: InMemoryDispersalSamplerNoError<M, H, G>>
    InMemoryDispersalSampler<M, H, G> for T
{
    /// Creates a new `T` from the `dispersal` map and extent of the habitat
    /// map.
    ///
    /// # Errors
    ///
    /// `Err(InconsistentDispersalMapSize)` is returned iff the dimensions of
    /// `dispersal` are not `ExE` given `E=WxH` where habitat has width `W`
    /// and height `W`.
    ///
    /// `Err(InconsistentDispersalProbabilities)` is returned iff any of the
    /// following conditions is violated:
    /// - habitat cells must disperse somewhere
    /// - non-habitat cells must not disperse
    /// - dispersal must only target habitat cells
    fn new(
        dispersal: &Array2D<NonNegativeF64>,
        habitat: &H,
    ) -> Result<Self, InMemoryDispersalSamplerError> {
        let habitat_extent = habitat.get_extent();

        let habitat_area =
            usize::from(habitat_extent.width()) * usize::from(habitat_extent.height());

        if dispersal.num_rows() != habitat_area || dispersal.num_columns() != habitat_area {
            return Err(InMemoryDispersalSamplerError::InconsistentDispersalMapSize);
        }

        if !explicit_in_memory_dispersal_check_contract(dispersal, habitat) {
            return Err(InMemoryDispersalSamplerError::InconsistentDispersalProbabilities);
        }

        Ok(Self::unchecked_new(dispersal, habitat))
    }
}
