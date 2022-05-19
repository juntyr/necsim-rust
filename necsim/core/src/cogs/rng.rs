use core::{convert::AsMut, ptr::copy_nonoverlapping};

use serde::{de::DeserializeOwned, Serialize};

use crate::{
    cogs::{Distribution, DistributionSampler, Habitat, MathsCore},
    landscape::IndexedLocation,
};

pub trait Rng<M: MathsCore>: RngCore {
    type Generator: RngCore;
    type Sampler;

    #[must_use]
    fn generator(&mut self) -> &mut Self::Generator;

    #[must_use]
    fn map_generator<F: FnOnce(Self::Generator) -> Self::Generator>(self, map: F) -> Self;

    fn with_rng<F: FnOnce(&mut Self::Generator, &Self::Sampler) -> Q, Q>(&mut self, inner: F) -> Q;
}

#[allow(clippy::module_name_repetitions)]
pub trait RngCore:
    crate::cogs::Backup + Sized + Clone + core::fmt::Debug + Serialize + DeserializeOwned
{
    type Seed: AsMut<[u8]> + Default + Sized;

    #[must_use]
    fn from_seed(seed: Self::Seed) -> Self;

    #[must_use]
    fn sample_u64(&mut self) -> u64;
}

#[allow(clippy::module_name_repetitions)]
pub trait SeedableRng: RngCore {
    #[must_use]
    fn seed_from_u64(mut state: u64) -> Self {
        // Implementation from:
        // https://docs.rs/rand/0.7.3/rand/trait.SeedableRng.html#method.seed_from_u64

        // We use PCG32 to generate a u32 sequence, and copy to the seed
        const MUL: u64 = 6_364_136_223_846_793_005_u64;
        const INC: u64 = 11_634_580_027_462_260_723_u64;

        let mut seed = Self::Seed::default();

        for chunk in seed.as_mut().chunks_mut(4) {
            // We advance the state first (to get away from the input value,
            // in case it has low Hamming Weight).
            state = state.wrapping_mul(MUL).wrapping_add(INC);

            // Use PCG output function with to_le to generate x:
            #[allow(clippy::cast_possible_truncation)]
            let xorshifted = (((state >> 18) ^ state) >> 27) as u32;
            #[allow(clippy::cast_possible_truncation)]
            let rot = (state >> 59) as u32;
            let x = xorshifted.rotate_right(rot).to_le();

            unsafe {
                let p = core::ptr::addr_of!(x).cast::<u8>();
                copy_nonoverlapping(p, chunk.as_mut_ptr(), chunk.len());
            }
        }

        Self::from_seed(seed)
    }
}

impl<R: RngCore> SeedableRng for R {}

#[allow(clippy::module_name_repetitions)]
pub trait PrimeableRng: RngCore {
    fn prime_with(&mut self, location_index: u64, time_index: u64);
}

#[allow(clippy::module_name_repetitions)]
pub trait HabitatPrimeableRng<M: MathsCore, H: Habitat<M>>: PrimeableRng {
    #[inline]
    fn prime_with_habitat(
        &mut self,
        habitat: &H,
        indexed_location: &IndexedLocation,
        time_index: u64,
    ) {
        self.prime_with(
            habitat.map_indexed_location_to_u64_injective(indexed_location),
            time_index,
        );
    }
}

impl<M: MathsCore, H: Habitat<M>, R: PrimeableRng> HabitatPrimeableRng<M, H> for R {}

#[allow(clippy::module_name_repetitions)]
pub trait SplittableRng: RngCore {
    #[must_use]
    fn split(self) -> (Self, Self);

    #[must_use]
    fn split_to_stream(self, stream: u64) -> Self;
}

pub trait Samples<M: MathsCore, D: Distribution + ?Sized>: Rng<M> {
    #[must_use]
    fn sample_with(&mut self, params: D::Parameters) -> D::Sample;

    #[must_use]
    fn sample(&mut self) -> D::Sample
    where
        D: Distribution<Parameters = ()>,
    {
        self.sample_with(())
    }
}

impl<M: MathsCore, D: Distribution, R: Rng<M>> Samples<M, D> for R
where
    R::Sampler: DistributionSampler<M, R::Generator, R::Sampler, D>,
{
    #[must_use]
    fn sample_with(&mut self, params: D::Parameters) -> D::Sample {
        self.with_rng(|rng, samplers| samplers.sample_with(rng, samplers, params))
    }
}
