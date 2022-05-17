use core::marker::PhantomData;

use necsim_core::{
    cogs::{Backup, MathsCore, Rng, UniformlySampleableHabitat},
    landscape::Location,
};

use super::AntiTrespassingDispersalSampler;

#[allow(clippy::module_name_repetitions)]
#[derive(Debug)]
#[cfg_attr(feature = "cuda", derive(rust_cuda::common::LendRustToCuda))]
#[cfg_attr(feature = "cuda", cuda(free = "M", free = "H", free = "G"))]
pub struct UniformAntiTrespassingDispersalSampler<
    M: MathsCore,
    H: UniformlySampleableHabitat<M, G>,
    G: Rng<M>,
> {
    marker: PhantomData<(M, H, G)>,
}

impl<M: MathsCore, H: UniformlySampleableHabitat<M, G>, G: Rng<M>> Default
    for UniformAntiTrespassingDispersalSampler<M, H, G>
{
    #[must_use]
    fn default() -> Self {
        Self {
            marker: PhantomData::<(M, H, G)>,
        }
    }
}

#[contract_trait]
impl<M: MathsCore, H: UniformlySampleableHabitat<M, G>, G: Rng<M>> Backup
    for UniformAntiTrespassingDispersalSampler<M, H, G>
{
    unsafe fn backup_unchecked(&self) -> Self {
        Self {
            marker: PhantomData::<(M, H, G)>,
        }
    }
}

#[contract_trait]
impl<M: MathsCore, H: UniformlySampleableHabitat<M, G>, G: Rng<M>>
    AntiTrespassingDispersalSampler<M, H, G> for UniformAntiTrespassingDispersalSampler<M, H, G>
{
    #[must_use]
    fn sample_anti_trespassing_dispersal_from_location(
        &self,
        _location: &Location,
        habitat: &H,
        rng: &mut G,
    ) -> Location {
        habitat.sample_habitable_indexed_location(rng).into()
    }
}
