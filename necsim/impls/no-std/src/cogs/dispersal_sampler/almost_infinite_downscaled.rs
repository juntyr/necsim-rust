use core::marker::PhantomData;

use necsim_core::{
    cogs::{DispersalSampler, MathsCore, RngCore, RngSampler, SeparableDispersalSampler},
    landscape::Location,
};
use necsim_core_bond::{ClosedUnitF64, NonNegativeF64, PositiveF64};

use crate::cogs::habitat::almost_infinite::AlmostInfiniteHabitat;

#[allow(clippy::module_name_repetitions)]
#[derive(Debug)]
#[cfg_attr(feature = "cuda", derive(rust_cuda::lend::LendRustToCuda))]
#[cfg_attr(feature = "cuda", cuda(free = "M", free = "G"))]
pub struct AlmostInfiniteDownscaledDispersalSampler<M: MathsCore, G: RngCore<M>, D: DispersalSampler<M, AlmostInfiniteHabitat<M>, G>> {
    #[cuda(embed)]
    dispersal: D,
    marker: PhantomData<(M, G)>,
}

impl<M: MathsCore, G: RngCore<M>, D: DispersalSampler<M, AlmostInfiniteHabitat<M>, G>> AlmostInfiniteDownscaledDispersalSampler<M, G, D> {
    #[must_use]
    pub fn new(dispersal: D) -> Self {
        Self {
            dispersal,
            marker: PhantomData::<(M, G)>,
        }
    }
}

impl<M: MathsCore, G: RngCore<M>> Clone for AlmostInfiniteDownscaledDispersalSampler<M, G> {
    fn clone(&self) -> Self {
        Self {
            shape_u: self.shape_u,
            tail_p: self.tail_p,
            self_dispersal: self.self_dispersal,
            marker: PhantomData::<(M, G)>,
        }
    }
}

#[contract_trait]
impl<M: MathsCore, G: RngCore<M>> DispersalSampler<M, AlmostInfiniteHabitat<M>, G>
    for AlmostInfiniteDownscaledDispersalSampler<M, G>
{
    #[must_use]
    fn sample_dispersal_from_location(
        &self,
        location: &Location,
        _habitat: &AlmostInfiniteHabitat<M>,
        rng: &mut G,
    ) -> Location {
        let jump =
            clark2dt::cdf_inverse::<M>(rng.sample_uniform_closed_open(), self.shape_u, self.tail_p);
        let theta = rng.sample_uniform_open_closed().get() * 2.0 * core::f64::consts::PI;

        let dx = M::cos(theta) * jump;
        let dy = M::sin(theta) * jump;

        AlmostInfiniteHabitat::<M>::clamp_round_dispersal(location, dx, dy)
    }
}

#[contract_trait]
impl<M: MathsCore, G: RngCore<M>> SeparableDispersalSampler<M, AlmostInfiniteHabitat<M>, G>
    for AlmostInfiniteDownscaledDispersalSampler<M, G>
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
        self.self_dispersal
    }
}
