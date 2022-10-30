use core::marker::PhantomData;

use necsim_core::{
    cogs::{Backup, DispersalSampler, Habitat, MathsCore, RngCore, SeparableDispersalSampler},
    landscape::Location,
};
use necsim_core_bond::ClosedUnitF64;

pub mod uniform;

#[allow(clippy::inline_always, clippy::inline_fn_without_body)]
#[allow(clippy::no_effect_underscore_binding)]
#[allow(clippy::module_name_repetitions)]
#[contract_trait]
pub trait AntiTrespassingDispersalSampler<M: MathsCore, H: Habitat<M>, G: RngCore<M>>:
    Backup + core::fmt::Debug
{
    #[must_use]
    #[debug_requires(!habitat.contains(location), "location is outside habitat")]
    #[debug_ensures(old(habitat).contains(&ret), "target is inside habitat")]
    fn sample_anti_trespassing_dispersal_from_location(
        &self,
        location: &Location,
        habitat: &H,
        rng: &mut G,
    ) -> Location;
}

#[allow(clippy::module_name_repetitions)]
#[derive(Debug)]
#[cfg_attr(feature = "cuda", derive(rust_cuda::common::LendRustToCuda))]
#[cfg_attr(feature = "cuda", cuda(free = "M"))]
pub struct TrespassingDispersalSampler<
    M: MathsCore,
    H: Habitat<M>,
    G: RngCore<M>,
    D: DispersalSampler<M, H, G>,
    T: AntiTrespassingDispersalSampler<M, H, G>,
> {
    #[cfg_attr(feature = "cuda", cuda(embed))]
    legal_dispersal_sampler: D,
    #[cfg_attr(feature = "cuda", cuda(embed))]
    trespassing_dispersal_sampler: T,
    marker: PhantomData<(M, H, G)>,
}

impl<
        M: MathsCore,
        H: Habitat<M>,
        G: RngCore<M>,
        D: DispersalSampler<M, H, G>,
        T: AntiTrespassingDispersalSampler<M, H, G>,
    > TrespassingDispersalSampler<M, H, G, D, T>
{
    #[must_use]
    pub fn new(dispersal_sampler: D, anti_trespassing: T) -> Self {
        Self {
            legal_dispersal_sampler: dispersal_sampler,
            trespassing_dispersal_sampler: anti_trespassing,
            marker: PhantomData::<(M, H, G)>,
        }
    }
}

#[contract_trait]
impl<
        M: MathsCore,
        H: Habitat<M>,
        G: RngCore<M>,
        D: DispersalSampler<M, H, G>,
        T: AntiTrespassingDispersalSampler<M, H, G>,
    > Backup for TrespassingDispersalSampler<M, H, G, D, T>
{
    unsafe fn backup_unchecked(&self) -> Self {
        Self {
            legal_dispersal_sampler: self.legal_dispersal_sampler.backup_unchecked(),
            trespassing_dispersal_sampler: self.trespassing_dispersal_sampler.backup_unchecked(),
            marker: PhantomData::<(M, H, G)>,
        }
    }
}

impl<
        M: MathsCore,
        H: Habitat<M>,
        G: RngCore<M>,
        D: DispersalSampler<M, H, G>,
        T: AntiTrespassingDispersalSampler<M, H, G>,
    > DispersalSampler<M, H, G> for TrespassingDispersalSampler<M, H, G, D, T>
{
    #[must_use]
    #[inline]
    #[allow(clippy::no_effect_underscore_binding)]
    #[debug_ensures(old(habitat).contains(&ret), "target is inside habitat")]
    fn sample_dispersal_from_location(
        &self,
        location: &Location,
        habitat: &H,
        rng: &mut G,
    ) -> Location {
        // Explicitly circumvent the precondition that `habitat.contains(location)`
        self.__contracts_impl_sample_dispersal_from_location(location, habitat, rng)
    }

    #[must_use]
    #[inline]
    fn __contracts_impl_sample_dispersal_from_location(
        &self,
        location: &Location,
        habitat: &H,
        rng: &mut G,
    ) -> Location {
        if habitat.contains(location) {
            // Re-establish the precondition that `habitat.contains(location)`
            self.legal_dispersal_sampler
                .sample_dispersal_from_location(location, habitat, rng)
        } else {
            // Establish the precondition that `!habitat.contains(location)`
            self.trespassing_dispersal_sampler
                .sample_anti_trespassing_dispersal_from_location(location, habitat, rng)
        }
    }
}

impl<
        M: MathsCore,
        H: Habitat<M>,
        G: RngCore<M>,
        D: SeparableDispersalSampler<M, H, G>,
        T: AntiTrespassingDispersalSampler<M, H, G>,
    > SeparableDispersalSampler<M, H, G> for TrespassingDispersalSampler<M, H, G, D, T>
{
    #[must_use]
    #[inline]
    #[allow(clippy::no_effect_underscore_binding)]
    #[debug_ensures(old(habitat).contains(&ret), "target is inside habitat")]
    #[debug_ensures(&ret != location, "disperses to a different location")]
    fn sample_non_self_dispersal_from_location(
        &self,
        location: &Location,
        habitat: &H,
        rng: &mut G,
    ) -> Location {
        // Explicitly circumvent the precondition that `habitat.contains(location)`
        self.__contracts_impl_sample_non_self_dispersal_from_location(location, habitat, rng)
    }

    #[must_use]
    #[inline]
    fn __contracts_impl_sample_non_self_dispersal_from_location(
        &self,
        location: &Location,
        habitat: &H,
        rng: &mut G,
    ) -> Location {
        if habitat.contains(location) {
            // Re-establish the precondition that `habitat.contains(location)`
            self.legal_dispersal_sampler
                .sample_non_self_dispersal_from_location(location, habitat, rng)
        } else {
            // Establish the precondition that `!habitat.contains(location)`
            self.trespassing_dispersal_sampler
                .sample_anti_trespassing_dispersal_from_location(location, habitat, rng)
        }
    }

    #[must_use]
    #[inline]
    fn get_self_dispersal_probability_at_location(
        &self,
        location: &Location,
        habitat: &H,
    ) -> ClosedUnitF64 {
        // Explicitly circumvent the precondition that `habitat.contains(location)`
        self.__contracts_impl_get_self_dispersal_probability_at_location(location, habitat)
    }

    #[must_use]
    #[inline]
    fn __contracts_impl_get_self_dispersal_probability_at_location(
        &self,
        location: &Location,
        habitat: &H,
    ) -> ClosedUnitF64 {
        if habitat.contains(location) {
            // Re-establish the precondition that `habitat.contains(location)`
            self.legal_dispersal_sampler
                .get_self_dispersal_probability_at_location(location, habitat)
        } else {
            // The `AntiTrespassingDispersalSampler` always jumps from outside
            //  the habitat to inside the habitat, i.e. there is never any
            //  self-dispersal outside the habitat
            ClosedUnitF64::zero()
        }
    }
}
