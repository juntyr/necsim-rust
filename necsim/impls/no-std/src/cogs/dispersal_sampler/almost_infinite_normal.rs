use core::marker::PhantomData;

use necsim_core::{
    cogs::{Backup, DispersalSampler, Habitat, RngCore, SeparableDispersalSampler},
    intrinsics::round,
    landscape::Location,
};
use necsim_core_bond::{NonNegativeF64, ZeroInclOneInclF64};

use crate::cogs::habitat::almost_infinite::AlmostInfiniteHabitat;

#[allow(clippy::module_name_repetitions)]
#[derive(Debug)]
#[cfg_attr(feature = "cuda", derive(RustToCuda))]
pub struct AlmostInfiniteNormalDispersalSampler<G: RngCore> {
    sigma: NonNegativeF64,
    self_dispersal: ZeroInclOneInclF64,
    marker: PhantomData<G>,
}

impl<G: RngCore> AlmostInfiniteNormalDispersalSampler<G> {
    #[must_use]
    pub fn new(sigma: NonNegativeF64) -> Self {
        let self_dispersal_1d = if sigma > 0.0_f64 {
            let probability = libm::erf(0.5 / (sigma.get() * core::f64::consts::SQRT_2));

            // Safety: For non-negative values x (as both sigma and sqrt(2.0) are),
            //         erf(0.5 / x) in [0.0; 1.0]
            unsafe { ZeroInclOneInclF64::new_unchecked(probability) }
        } else {
            ZeroInclOneInclF64::one()
        };

        Self {
            sigma,
            self_dispersal: self_dispersal_1d * self_dispersal_1d,
            marker: PhantomData::<G>,
        }
    }
}

#[contract_trait]
impl<G: RngCore> Backup for AlmostInfiniteNormalDispersalSampler<G> {
    unsafe fn backup_unchecked(&self) -> Self {
        Self {
            sigma: self.sigma,
            self_dispersal: self.self_dispersal,
            marker: PhantomData::<G>,
        }
    }
}

#[contract_trait]
impl<G: RngCore> DispersalSampler<AlmostInfiniteHabitat, G>
    for AlmostInfiniteNormalDispersalSampler<G>
{
    #[must_use]
    fn sample_dispersal_from_location(
        &self,
        location: &Location,
        habitat: &AlmostInfiniteHabitat,
        rng: &mut G,
    ) -> Location {
        use necsim_core::cogs::RngSampler;

        let (dx, dy): (f64, f64) = rng.sample_2d_normal(0.0_f64, self.sigma);

        // Discrete dispersal assumes lineage positions are centred on (0.5, 0.5),
        // i.e. |dispersal| >= 0.5 changes the cell
        // (dx and dy must be rounded to nearest int away from 0.0)
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let (dx, dy): (i64, i64) = (
            (round(dx) as i64) % i64::from(habitat.get_extent().width()),
            (round(dy) as i64) % i64::from(habitat.get_extent().height()),
        );

        let new_x = (i64::from(location.x()) + dx) % i64::from(habitat.get_extent().width());
        let new_y = (i64::from(location.y()) + dy) % i64::from(habitat.get_extent().height());

        #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
        Location::new(
            ((new_x + i64::from(habitat.get_extent().width()))
                % i64::from(habitat.get_extent().width())) as u32,
            ((new_y + i64::from(habitat.get_extent().height()))
                % i64::from(habitat.get_extent().height())) as u32,
        )
    }
}

#[contract_trait]
impl<G: RngCore> SeparableDispersalSampler<AlmostInfiniteHabitat, G>
    for AlmostInfiniteNormalDispersalSampler<G>
{
    #[must_use]
    fn sample_non_self_dispersal_from_location(
        &self,
        location: &Location,
        habitat: &AlmostInfiniteHabitat,
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
        _habitat: &AlmostInfiniteHabitat,
    ) -> ZeroInclOneInclF64 {
        self.self_dispersal
    }
}
