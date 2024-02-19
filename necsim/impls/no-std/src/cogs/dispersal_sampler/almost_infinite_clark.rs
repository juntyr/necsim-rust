use core::marker::PhantomData;

use necsim_core::{
    cogs::{Backup, DispersalSampler, MathsCore, RngCore, SeparableDispersalSampler},
    landscape::Location,
};
use necsim_core_bond::{ClosedUnitF64, PositiveF64};

use crate::cogs::habitat::almost_infinite::AlmostInfiniteHabitat;

#[allow(clippy::module_name_repetitions)]
#[derive(Debug)]
#[cfg_attr(feature = "cuda", derive(rust_cuda::common::LendRustToCuda))]
#[cfg_attr(feature = "cuda", cuda(free = "M", free = "G"))]
pub struct AlmostInfiniteClarkDispersalSampler<M: MathsCore, G: RngCore<M>> {
    shape: PositiveF64,
    marker: PhantomData<(M, G)>,
}

impl<M: MathsCore, G: RngCore<M>> AlmostInfiniteClarkDispersalSampler<M, G> {
    #[must_use]
    pub fn new(shape: PositiveF64) -> Self {
        Self {
            shape,
            marker: PhantomData::<(M, G)>,
        }
    }
}

#[contract_trait]
impl<M: MathsCore, G: RngCore<M>> Backup for AlmostInfiniteClarkDispersalSampler<M, G> {
    unsafe fn backup_unchecked(&self) -> Self {
        Self {
            shape: self.shape,
            marker: PhantomData::<(M, G)>,
        }
    }
}

#[contract_trait]
impl<M: MathsCore, G: RngCore<M>> DispersalSampler<M, AlmostInfiniteHabitat<M>, G>
    for AlmostInfiniteClarkDispersalSampler<M, G>
{
    #[must_use]
    fn sample_dispersal_from_location(
        &self,
        _location: &Location,
        _habitat: &AlmostInfiniteHabitat<M>,
        _rng: &mut G,
    ) -> Location {
        todo!()
    }
}

#[contract_trait]
impl<M: MathsCore, G: RngCore<M>> SeparableDispersalSampler<M, AlmostInfiniteHabitat<M>, G>
    for AlmostInfiniteClarkDispersalSampler<M, G>
{
    #[must_use]
    fn sample_non_self_dispersal_from_location(
        &self,
        location: &Location,
        habitat: &AlmostInfiniteHabitat<M>,
        rng: &mut G,
    ) -> Location {
        let mut target_location = self.sample_dispersal_from_location(location, habitat, rng);

        // For now, we just use rejection sampling here
        while &target_location == location {
            target_location = self.sample_dispersal_from_location(location, habitat, rng);
        }

        target_location
    }

    #[must_use]
    fn get_self_dispersal_probability_at_location(
        &self,
        _location: &Location,
        _habitat: &AlmostInfiniteHabitat<M>,
    ) -> ClosedUnitF64 {
        todo!()
    }
}
