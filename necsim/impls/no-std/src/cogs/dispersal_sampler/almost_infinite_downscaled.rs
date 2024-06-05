use core::marker::PhantomData;

use necsim_core::{
    cogs::{DispersalSampler, MathsCore, RngCore, RngSampler, SeparableDispersalSampler},
    landscape::Location,
};
use necsim_core_bond::ClosedUnitF64;

use crate::cogs::habitat::almost_infinite::{
    downscaled::AlmostInfiniteDownscaledHabitat, AlmostInfiniteHabitat,
};

#[allow(clippy::module_name_repetitions)]
#[derive(Debug)]
#[cfg_attr(feature = "cuda", derive(rust_cuda::lend::LendRustToCuda))]
#[cfg_attr(feature = "cuda", cuda(free = "M", free = "G"))]
pub struct AlmostInfiniteDownscaledDispersalSampler<
    M: MathsCore,
    G: RngCore<M>,
    D: Clone + DispersalSampler<M, AlmostInfiniteHabitat<M>, G>,
> {
    #[cfg_attr(feature = "cuda", cuda(embed))]
    dispersal: D,
    marker: PhantomData<(M, G)>,
}

impl<M: MathsCore, G: RngCore<M>, D: Clone + DispersalSampler<M, AlmostInfiniteHabitat<M>, G>>
    AlmostInfiniteDownscaledDispersalSampler<M, G, D>
{
    #[must_use]
    pub fn new(dispersal: D) -> Self {
        Self {
            dispersal,
            marker: PhantomData::<(M, G)>,
        }
    }
}

impl<M: MathsCore, G: RngCore<M>, D: Clone + DispersalSampler<M, AlmostInfiniteHabitat<M>, G>> Clone
    for AlmostInfiniteDownscaledDispersalSampler<M, G, D>
{
    fn clone(&self) -> Self {
        Self {
            dispersal: self.dispersal.clone(),
            marker: PhantomData::<(M, G)>,
        }
    }
}

#[contract_trait]
impl<M: MathsCore, G: RngCore<M>, D: Clone + DispersalSampler<M, AlmostInfiniteHabitat<M>, G>>
    DispersalSampler<M, AlmostInfiniteDownscaledHabitat<M>, G>
    for AlmostInfiniteDownscaledDispersalSampler<M, G, D>
{
    #[must_use]
    fn sample_dispersal_from_location(
        &self,
        location: &Location,
        habitat: &AlmostInfiniteDownscaledHabitat<M>,
        rng: &mut G,
    ) -> Location {
        // TODO: optimise
        let sub_index = rng.sample_index_u32(habitat.downscale_area());

        let index_x = sub_index % (habitat.downscale_x() as u32);
        let index_y = sub_index % (habitat.downscale_y() as u32);

        // generate an upscaled location by sampling a random sub-location
        let location = Location::new(location.x() + index_x, location.y() + index_y);

        // sample dispersal from the inner dispersal sampler as normal
        let target_location =
            self.dispersal
                .sample_dispersal_from_location(&location, habitat.unscaled(), rng);

        let index_x = target_location.x() % (habitat.downscale_x() as u32);
        let index_y = target_location.y() % (habitat.downscale_y() as u32);

        // downscale the target location
        Location::new(target_location.x() - index_x, target_location.y() - index_y)
    }
}

#[contract_trait]
impl<M: MathsCore, G: RngCore<M>, D: Clone + DispersalSampler<M, AlmostInfiniteHabitat<M>, G>>
    SeparableDispersalSampler<M, AlmostInfiniteDownscaledHabitat<M>, G>
    for AlmostInfiniteDownscaledDispersalSampler<M, G, D>
{
    #[must_use]
    fn sample_non_self_dispersal_from_location(
        &self,
        location: &Location,
        habitat: &AlmostInfiniteDownscaledHabitat<M>,
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
        _habitat: &AlmostInfiniteDownscaledHabitat<M>,
    ) -> ClosedUnitF64 {
        unimplemented!()
    }
}
