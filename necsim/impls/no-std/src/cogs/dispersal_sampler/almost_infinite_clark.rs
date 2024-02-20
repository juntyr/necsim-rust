use core::marker::PhantomData;

use necsim_core::{
    cogs::{Backup, DispersalSampler, MathsCore, RngCore, RngSampler, SeparableDispersalSampler},
    landscape::Location,
};
use necsim_core_bond::{ClosedUnitF64, NonNegativeF64, OpenClosedUnitF64, PositiveF64};

use crate::cogs::habitat::almost_infinite::AlmostInfiniteHabitat;

#[allow(clippy::module_name_repetitions)]
#[derive(Debug)]
#[cfg_attr(feature = "cuda", derive(rust_cuda::common::LendRustToCuda))]
#[cfg_attr(feature = "cuda", cuda(free = "M", free = "G"))]
pub struct AlmostInfiniteClarkDispersalSampler<M: MathsCore, G: RngCore<M>> {
    shape_u: PositiveF64,
    self_dispersal: ClosedUnitF64,
    marker: PhantomData<(M, G)>,
}

impl<M: MathsCore, G: RngCore<M>> AlmostInfiniteClarkDispersalSampler<M, G> {
    #[must_use]
    pub fn new(shape_u: PositiveF64) -> Self {
        const N: i32 = 1 << 22;

        // For now, we numerically integrate the self-dispersal probability
        //  using polar coordinates
        #[allow(clippy::useless_conversion)] // prepare for new range iterators
        let self_dispersal = (0..N)
            .into_iter()
            .map(|i| {
                // phi in [0, pi/4]
                core::f64::consts::PI * 0.25 * f64::from(i) / f64::from(N)
            })
            .map(|phi| {
                // self-dispersal jump radius: dx <= 0.5 && dy <= 0.5
                let jump_r = 0.5 / M::cos(phi);
                // Safety: cos([0, pi/4]) in [sqrt(2)/2, 1], and its inverse is non-negative
                unsafe { NonNegativeF64::new_unchecked(jump_r) }
            })
            .map(|jump_r| {
                // probability of dispersal to a jump distance <= jump_r
                clark2dt_cdf_p1(jump_r, shape_u).get()
            })
            .sum::<f64>()
            / f64::from(N); // take the average

        // Safety: the average of the cdfs, which are all ClosedUnitF64, is also in [0,
        // 1] Note: we still clamp to account for rounding errors
        let self_dispersal =
            unsafe { ClosedUnitF64::new_unchecked(self_dispersal.clamp(0.0, 1.0)) };

        Self {
            shape_u,
            self_dispersal,
            marker: PhantomData::<(M, G)>,
        }
    }
}

#[contract_trait]
impl<M: MathsCore, G: RngCore<M>> Backup for AlmostInfiniteClarkDispersalSampler<M, G> {
    unsafe fn backup_unchecked(&self) -> Self {
        Self {
            shape_u: self.shape_u,
            self_dispersal: self.self_dispersal,
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
        self.self_dispersal
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

fn clark2dt_cdf_p1(jump_r: NonNegativeF64, shape_u: PositiveF64) -> ClosedUnitF64 {
    // assume p (tail width) = 1
    let u01 = 1.0 - (1.0 / (1.0 + jump_r.get() * jump_r.get() / shape_u.get()));

    unsafe { ClosedUnitF64::new_unchecked(u01) }
}
