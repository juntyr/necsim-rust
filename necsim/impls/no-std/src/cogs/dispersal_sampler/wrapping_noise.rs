use necsim_core::{
    cogs::{Backup, DispersalSampler, Habitat, MathsCore, RngCore},
    landscape::Location,
};
use necsim_core_bond::NonNegativeF64;

use crate::cogs::{
    dispersal_sampler::almost_infinite_normal::AlmostInfiniteNormalDispersalSampler,
    habitat::wrapping_noise::WrappingNoiseHabitat,
};

#[allow(clippy::module_name_repetitions)]
#[derive(Debug)]
#[cfg_attr(feature = "cuda", derive(rust_cuda::common::LendRustToCuda))]
#[cfg_attr(feature = "cuda", cuda(free = "M"))]
pub struct WrappingNoiseNormalDispersalSampler<M: MathsCore, G: RngCore<M>> {
    #[cfg_attr(feature = "cuda", cuda(embed))]
    inner: AlmostInfiniteNormalDispersalSampler<M, G>,
}

impl<M: MathsCore, G: RngCore<M>> WrappingNoiseNormalDispersalSampler<M, G> {
    #[must_use]
    pub fn new(sigma: NonNegativeF64) -> Self {
        Self {
            inner: AlmostInfiniteNormalDispersalSampler::new(sigma),
        }
    }
}

#[contract_trait]
impl<M: MathsCore, G: RngCore<M>> Backup for WrappingNoiseNormalDispersalSampler<M, G> {
    unsafe fn backup_unchecked(&self) -> Self {
        Self {
            inner: self.inner.backup_unchecked(),
        }
    }
}

#[contract_trait]
impl<M: MathsCore, G: RngCore<M>> DispersalSampler<M, WrappingNoiseHabitat<M>, G>
    for WrappingNoiseNormalDispersalSampler<M, G>
{
    #[must_use]
    #[inline]
    fn sample_dispersal_from_location(
        &self,
        location: &Location,
        habitat: &WrappingNoiseHabitat<M>,
        rng: &mut G,
    ) -> Location {
        let mut target =
            self.inner
                .sample_dispersal_from_location(location, habitat.get_inner(), rng);

        // Rejection sample the normal dispersal kernel
        while habitat.get_habitat_at_location(&target) == 0_u32 {
            target = self
                .inner
                .sample_dispersal_from_location(location, habitat.get_inner(), rng);
        }

        target
    }
}
