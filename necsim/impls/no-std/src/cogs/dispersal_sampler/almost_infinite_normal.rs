use core::marker::PhantomData;

use necsim_core::{
    cogs::{Backup, DispersalSampler, MathsCore, RngCore, SeparableDispersalSampler},
    landscape::Location,
};
use necsim_core_bond::{ClosedUnitF64, NonNegativeF64};

use crate::cogs::habitat::almost_infinite::AlmostInfiniteHabitat;

#[allow(clippy::module_name_repetitions)]
#[derive(Debug)]
#[cfg_attr(feature = "cuda", derive(rust_cuda::common::LendRustToCuda))]
#[cfg_attr(feature = "cuda", cuda(layout::free = "M"))]
#[cfg_attr(feature = "cuda", cuda(layout::free = "G"))]
pub struct AlmostInfiniteNormalDispersalSampler<M: MathsCore, G: RngCore<M>> {
    sigma: NonNegativeF64,
    self_dispersal: ClosedUnitF64,
    marker: PhantomData<(M, G)>,
}

impl<M: MathsCore, G: RngCore<M>> AlmostInfiniteNormalDispersalSampler<M, G> {
    #[must_use]
    pub fn new(sigma: NonNegativeF64) -> Self {
        let self_dispersal_1d = if sigma > 0.0_f64 {
            let probability = libm::erf(0.5 / (sigma.get() * core::f64::consts::SQRT_2));

            // Safety: For non-negative values x (as both sigma and sqrt(2.0) are),
            //         erf(0.5 / x) in [0.0; 1.0]
            unsafe { ClosedUnitF64::new_unchecked(probability) }
        } else {
            ClosedUnitF64::one()
        };

        Self {
            sigma,
            self_dispersal: self_dispersal_1d * self_dispersal_1d,
            marker: PhantomData::<(M, G)>,
        }
    }
}

#[contract_trait]
impl<M: MathsCore, G: RngCore<M>> Backup for AlmostInfiniteNormalDispersalSampler<M, G> {
    unsafe fn backup_unchecked(&self) -> Self {
        Self {
            sigma: self.sigma,
            self_dispersal: self.self_dispersal,
            marker: PhantomData::<(M, G)>,
        }
    }
}

#[contract_trait]
impl<M: MathsCore, G: RngCore<M>> DispersalSampler<M, AlmostInfiniteHabitat<M>, G>
    for AlmostInfiniteNormalDispersalSampler<M, G>
{
    #[must_use]
    fn sample_dispersal_from_location(
        &self,
        location: &Location,
        _habitat: &AlmostInfiniteHabitat<M>,
        rng: &mut G,
    ) -> Location {
        use necsim_core::cogs::RngSampler;

        const WRAP: i64 = 1 << 32;

        let (dx, dy): (f64, f64) = rng.sample_2d_normal(0.0_f64, self.sigma);

        // Discrete dispersal assumes lineage positions are centred on (0.5, 0.5),
        // i.e. |dispersal| >= 0.5 changes the cell
        // (dx and dy must be rounded to nearest int away from 0.0)
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let (dx, dy): (i64, i64) = (M::round(dx) as i64, M::round(dy) as i64);

        let new_x = (i64::from(location.x()) + dx) % WRAP;
        let new_y = (i64::from(location.y()) + dy) % WRAP;

        #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
        Location::new(
            ((new_x + WRAP) % WRAP) as u32,
            ((new_y + WRAP) % WRAP) as u32,
        )
    }
}

#[contract_trait]
impl<M: MathsCore, G: RngCore<M>> SeparableDispersalSampler<M, AlmostInfiniteHabitat<M>, G>
    for AlmostInfiniteNormalDispersalSampler<M, G>
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
