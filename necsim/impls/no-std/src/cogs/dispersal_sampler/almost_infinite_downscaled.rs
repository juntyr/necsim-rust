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
    self_dispersal: ClosedUnitF64,
    marker: PhantomData<(M, G)>,
}

impl<M: MathsCore, G: RngCore<M>, D: Clone + DispersalSampler<M, AlmostInfiniteHabitat<M>, G>>
    AlmostInfiniteDownscaledDispersalSampler<M, G, D>
{
    #[must_use]
    pub fn new(habitat: &AlmostInfiniteDownscaledHabitat<M>, dispersal: D) -> Self {
        use necsim_core::cogs::SeedableRng;

        const N: i32 = 1 << 22;

        let dispersal = Self {
            dispersal,
            // since the dispersal sampler doesn't need to know its self-dispersal
            //  to perform non-separable dispersal, we can set it to zero here
            self_dispersal: ClosedUnitF64::zero(),
            marker: PhantomData::<(M, G)>,
        };

        let mut rng = G::seed_from_u64(42);
        let mut counter = 0_i32;

        let origin = Location::new(0, 0);

        // TODO: optimise
        for _ in 0..N {
            let target = dispersal.sample_dispersal_from_location(&origin, habitat, &mut rng);

            if target == origin {
                counter += 1;
            }
        }

        let self_dispersal_emperical = f64::from(counter) / f64::from(N);
        // Safety: the fraction of 0 >= counter <= N and N must be in [0.0; 1.0]
        // Note: we still clamp to account for rounding errors
        let self_dispersal_emperical =
            unsafe { ClosedUnitF64::new_unchecked(self_dispersal_emperical.clamp(0.0, 1.0)) };

        Self {
            dispersal: dispersal.dispersal,
            self_dispersal: self_dispersal_emperical,
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
            self_dispersal: self.self_dispersal,
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
        // // TODO: must be optimised
        // let mut target_location = self.sample_dispersal_from_location(location,
        // habitat, rng);

        // // For now, we just use rejection sampling here
        // while &target_location == location {
        //     target_location = self.sample_dispersal_from_location(location, habitat,
        // rng); }

        // target_location

        // very dirty nearest neighbour dispersal
        let direction = rng.sample_index(core::num::NonZeroUsize::MIN.saturating_add(7));

        // 0 1 2
        // 7   3
        // 6 5 4

        let x = match direction {
            0 | 6 | 7 => location.x().wrapping_sub(habitat.downscale_x() as u32),
            1 | 5 => location.x(),
            2..=4 => location.x().wrapping_add(habitat.downscale_x() as u32),
            _ => unreachable!(),
        };
        let y = match direction {
            0..=2 => location.y().wrapping_add(habitat.downscale_y() as u32),
            3 | 7 => location.y(),
            4..=6 => location.y().wrapping_sub(habitat.downscale_y() as u32),
            _ => unreachable!(),
        };

        Location::new(x, y)
    }

    #[must_use]
    fn get_self_dispersal_probability_at_location(
        &self,
        _location: &Location,
        _habitat: &AlmostInfiniteDownscaledHabitat<M>,
    ) -> ClosedUnitF64 {
        self.self_dispersal
    }
}
