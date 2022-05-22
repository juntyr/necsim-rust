use core::{fmt, marker::PhantomData};

use necsim_core::cogs::{Backup, MathsCore, Rng, RngCore};

use rand_core::{Error, RngCore as RandRngCore, SeedableRng as RandSeedableRng};
use serde::{de::DeserializeOwned, Deserialize, Deserializer, Serialize, Serializer};

use crate::cogs::distribution::rand::RandDistributionSamplers;

#[allow(clippy::module_name_repetitions)]
#[derive(TypeLayout)]
#[repr(transparent)]
pub struct RandAsRng<G: RandRngCore + RandSeedableRng + Clone + Serialize + DeserializeOwned> {
    inner: G,
}

impl<G: RandRngCore + RandSeedableRng + Clone + Serialize + DeserializeOwned> From<G>
    for RandAsRng<G>
{
    #[inline]
    fn from(inner: G) -> Self {
        Self { inner }
    }
}

impl<G: RandRngCore + RandSeedableRng + Clone + Serialize + DeserializeOwned> RandAsRng<G> {
    #[must_use]
    pub fn into_inner(self) -> G {
        self.inner
    }
}

impl<G: RandRngCore + RandSeedableRng + Clone + Serialize + DeserializeOwned> fmt::Debug
    for RandAsRng<G>
{
    default fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        struct InnerRng(&'static str);

        impl fmt::Debug for InnerRng {
            fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
                fmt.write_str(self.0)
            }
        }

        fmt.debug_tuple("RandAsRng")
            .field(&InnerRng(core::any::type_name::<G>()))
            .finish()
    }
}

impl<G: RandRngCore + RandSeedableRng + Clone + Serialize + DeserializeOwned + fmt::Debug>
    fmt::Debug for RandAsRng<G>
{
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_tuple("RandAsRng").field(&self.inner).finish()
    }
}

#[contract_trait]
impl<G: RandRngCore + RandSeedableRng + Clone + Serialize + DeserializeOwned> Backup
    for RandAsRng<G>
{
    unsafe fn backup_unchecked(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<G: RandRngCore + RandSeedableRng + Clone + Serialize + DeserializeOwned> Serialize
    for RandAsRng<G>
{
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.inner.serialize(serializer)
    }
}

impl<'de, G: RandRngCore + RandSeedableRng + Clone + Serialize + DeserializeOwned> Deserialize<'de>
    for RandAsRng<G>
{
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let inner = G::deserialize(deserializer)?;

        Ok(Self { inner })
    }
}

impl<G: RandRngCore + RandSeedableRng + Clone + Serialize + DeserializeOwned> RngCore
    for RandAsRng<G>
{
    type Seed = G::Seed;

    #[must_use]
    #[inline]
    fn from_seed(seed: Self::Seed) -> Self {
        Self {
            inner: G::from_seed(seed),
        }
    }

    #[must_use]
    #[inline]
    fn sample_u64(&mut self) -> u64 {
        self.inner.next_u64()
    }
}

#[allow(clippy::module_name_repetitions)]
#[derive(TypeLayout)]
#[repr(transparent)]
pub struct RngAsRand<G: RngCore> {
    inner: G,
}

impl<G: RngCore> From<G> for RngAsRand<G> {
    #[inline]
    fn from(inner: G) -> Self {
        Self { inner }
    }
}

impl<G: RngCore> RngAsRand<G> {
    #[must_use]
    pub fn into_inner(self) -> G {
        self.inner
    }
}

impl<G: RngCore> fmt::Debug for RngAsRand<G> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_tuple("RngAsRand").field(&self.inner).finish()
    }
}

#[contract_trait]
impl<G: RngCore> Backup for RngAsRand<G> {
    unsafe fn backup_unchecked(&self) -> Self {
        Self {
            inner: self.inner.backup_unchecked(),
        }
    }
}

impl<G: RngCore> Serialize for RngAsRand<G> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.inner.serialize(serializer)
    }
}

impl<'de, G: RngCore> Deserialize<'de> for RngAsRand<G> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let inner = G::deserialize(deserializer)?;

        Ok(Self { inner })
    }
}

impl<G: RngCore> RngCore for RngAsRand<G> {
    type Seed = G::Seed;

    #[must_use]
    #[inline]
    fn from_seed(seed: Self::Seed) -> Self {
        Self {
            inner: G::from_seed(seed),
        }
    }

    #[must_use]
    #[inline]
    fn sample_u64(&mut self) -> u64 {
        self.inner.sample_u64()
    }
}

impl<G: RngCore> RandSeedableRng for RngAsRand<G> {
    type Seed = G::Seed;

    #[inline]
    fn from_seed(seed: Self::Seed) -> Self {
        Self {
            inner: G::from_seed(seed),
        }
    }
}

impl<G: RngCore> RandRngCore for RngAsRand<G> {
    #[inline]
    default fn next_u32(&mut self) -> u32 {
        // Note: The most significant bits are often a bit more random
        (self.sample_u64() >> 32) as u32
    }

    #[inline]
    default fn next_u64(&mut self) -> u64 {
        self.sample_u64()
    }

    #[inline]
    default fn fill_bytes(&mut self, dest: &mut [u8]) {
        rand_core::impls::fill_bytes_via_next(self, dest);
    }

    #[inline]
    #[allow(clippy::unit_arg)]
    default fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), Error> {
        Ok(self.fill_bytes(dest))
    }
}

impl<G: RngCore + RandRngCore> RandRngCore for RngAsRand<G> {
    #[inline]
    fn next_u32(&mut self) -> u32 {
        self.inner.next_u32()
    }

    #[inline]
    fn next_u64(&mut self) -> u64 {
        self.inner.next_u64()
    }

    #[inline]
    fn fill_bytes(&mut self, dest: &mut [u8]) {
        self.inner.fill_bytes(dest);
    }

    #[inline]
    fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), Error> {
        self.inner.try_fill_bytes(dest)
    }
}

#[derive(Debug, TypeLayout)]
#[allow(clippy::module_name_repetitions)]
#[repr(transparent)]
pub struct RandRng<M: MathsCore, R: RngCore + RandRngCore> {
    inner: R,
    marker: PhantomData<M>,
}

impl<M: MathsCore, R: RngCore + RandRngCore> From<R> for RandRng<M, R> {
    fn from(inner: R) -> Self {
        Self {
            inner,
            marker: PhantomData::<M>,
        }
    }
}

impl<M: MathsCore, R: RngCore + RandRngCore> RandRng<M, R> {
    pub fn into_inner(self) -> R {
        self.inner
    }
}

#[contract_trait]
impl<M: MathsCore, R: RngCore + RandRngCore> Backup for RandRng<M, R> {
    unsafe fn backup_unchecked(&self) -> Self {
        Self {
            inner: self.inner.backup_unchecked(),
            marker: PhantomData::<M>,
        }
    }
}

impl<M: MathsCore, R: RngCore + RandRngCore> Rng<M> for RandRng<M, R> {
    type Generator = R;
    type Sampler = RandDistributionSamplers;

    fn generator(&mut self) -> &mut Self::Generator {
        &mut self.inner
    }

    fn map_generator<F: FnOnce(Self::Generator) -> Self::Generator>(self, map: F) -> Self {
        let RandRng { inner, marker } = self;

        RandRng {
            inner: map(inner),
            marker,
        }
    }

    fn with<F: FnOnce(&mut Self::Generator, &Self::Sampler) -> Q, Q>(&mut self, inner: F) -> Q {
        inner(&mut self.inner, &RandDistributionSamplers)
    }
}
