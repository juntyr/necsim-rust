use core::marker::PhantomData;

use necsim_core::{
    cogs::{Backup, DispersalSampler, MathsCore, RngCore, RngSampler, SeparableDispersalSampler},
    landscape::Location,
};
use necsim_core_bond::{ClosedUnitF64, OpenClosedUnitF64, PositiveF64};

use crate::cogs::habitat::almost_infinite::AlmostInfiniteHabitat;

#[allow(clippy::module_name_repetitions)]
#[derive(Debug)]
#[cfg_attr(feature = "cuda", derive(rust_cuda::common::LendRustToCuda))]
#[cfg_attr(feature = "cuda", cuda(free = "M", free = "G"))]
pub struct AlmostInfiniteClarkDispersalSampler<M: MathsCore, G: RngCore<M>> {
    shape_u: PositiveF64,
    marker: PhantomData<(M, G)>,
}

impl<M: MathsCore, G: RngCore<M>> AlmostInfiniteClarkDispersalSampler<M, G> {
    #[must_use]
    pub fn new(shape_u: PositiveF64) -> Self {
        Self {
            shape_u,
            marker: PhantomData::<(M, G)>,
        }
    }
}

#[contract_trait]
impl<M: MathsCore, G: RngCore<M>> Backup for AlmostInfiniteClarkDispersalSampler<M, G> {
    unsafe fn backup_unchecked(&self) -> Self {
        Self {
            shape_u: self.shape_u,
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
        location: &Location,
        _habitat: &AlmostInfiniteHabitat<M>,
        rng: &mut G,
    ) -> Location {
        let jump = clark2dt_cdf_inverse_p1::<M>(rng.sample_uniform_open_closed(), self.shape_u);
        let theta = rng.sample_uniform_open_closed().get() * 2.0 * core::f64::consts::PI;

        let dx = M::cos(theta) * jump;
        let dy = M::sin(theta) * jump;

        AlmostInfiniteHabitat::<M>::clamp_round_dispersal(location, dx, dy)
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

fn clark2dt_cdf_inverse_p1<M: MathsCore>(u01: OpenClosedUnitF64, shape_u: PositiveF64) -> f64 {
    // pdf(r) = (2 * pi * r) * p / (pi * u * (1 + r*r / u)) ** (p + 1)
    // pdf(r, p=1) = (2 * pi * r) / (pi * u * (1 + r*r / u)) ** 2

    // cdf(r) = 1 - (1 / (1 + r*r / u) ** p)
    // cdf(r, p=1) = 1 - (1 / (1 + r*r / u))

    // r = cdf_inv(u01) = sqrt(u * (((1 / (1 - u01)) ** (1/p)) - 1))
    // r = cdf_inv(u01, p=1) = sqrt(u * ((1 / (1 - u01)) - 1))

    // Note that 1-u01 ~ u01

    // See https://gist.github.com/juntyr/c04f231ba8063a336744f1e1359f40d8

    // assume p (tail width) = 1
    M::sqrt(shape_u.get() * ((1.0 / u01.get()) - 1.0))
}
