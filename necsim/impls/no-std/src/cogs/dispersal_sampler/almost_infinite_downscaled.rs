use core::marker::PhantomData;

use alloc::{sync::Arc, vec::Vec};
use hashbrown::HashMap;
use necsim_core::{
    cogs::{DispersalSampler, MathsCore, RngCore, RngSampler, SeparableDispersalSampler},
    landscape::Location,
};
use necsim_core_bond::{ClosedUnitF64, NonNegativeF64};

use crate::{
    alias::packed::AliasMethodSamplerAtom,
    cogs::habitat::almost_infinite::{
        downscaled::AlmostInfiniteDownscaledHabitat, AlmostInfiniteHabitat,
    },
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
    #[cfg_attr(feature = "cuda", cuda(embed))]
    non_self_dispersal: Arc<[AliasMethodSamplerAtom<(u32, u32)>]>,
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
            //  and non-self-dispersal to perform non-separable dispersal, we can
            //  set it to zero here
            self_dispersal: ClosedUnitF64::zero(),
            non_self_dispersal: Arc::from(Vec::new()),
            marker: PhantomData::<(M, G)>,
        };

        let origin = Location::new(0, 0);
        let mut rng = G::seed_from_u64(42);

        let mut targets = HashMap::new();

        // TODO: optimise
        for _ in 0..N {
            let target = dispersal.sample_dispersal_from_location(&origin, habitat, &mut rng);

            *targets.entry(target).or_insert(0_u32) += 1;
        }

        let self_dispersal_count = targets.get(&origin).copied().unwrap_or(0);
        let self_dispersal_emperical = f64::from(self_dispersal_count) / f64::from(N);
        // Safety: the fraction of 0 >= counter <= N and N must be in [0.0; 1.0]
        // Note: we still clamp to account for rounding errors
        let self_dispersal_emperical =
            unsafe { ClosedUnitF64::new_unchecked(self_dispersal_emperical.clamp(0.0, 1.0)) };

        let mut non_self_dispersal = targets
            .into_iter()
            .filter_map(|(target, count)| {
                if target == origin {
                    None
                } else {
                    Some(((target.x(), target.y()), NonNegativeF64::from(count)))
                }
            })
            .collect::<Vec<_>>();
        // sort to ensure reproducible construction of the non-self-dispersal alias
        //  sampler
        non_self_dispersal.sort_unstable();
        let non_self_dispersal = AliasMethodSamplerAtom::create(&non_self_dispersal);

        Self {
            dispersal: dispersal.dispersal,
            self_dispersal: self_dispersal_emperical,
            non_self_dispersal: Arc::from(non_self_dispersal),
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
            non_self_dispersal: self.non_self_dispersal.clone(),
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
        _habitat: &AlmostInfiniteDownscaledHabitat<M>,
        rng: &mut G,
    ) -> Location {
        // TODO: improve accuracy
        let (offset_x, offset_y) =
            AliasMethodSamplerAtom::sample_event(&self.non_self_dispersal, rng);

        Location::new(
            location.x().wrapping_add(offset_x),
            location.y().wrapping_add(offset_y),
        )
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
