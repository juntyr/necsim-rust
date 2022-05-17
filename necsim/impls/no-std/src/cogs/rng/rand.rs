use core::fmt;

use necsim_core::cogs::{Backup, RngCore};

use rand_core::{RngCore as RandRngCore, SeedableRng as RandSeedableRng};
use serde::{de::DeserializeOwned, Deserialize, Deserializer, Serialize, Serializer};

#[allow(clippy::module_name_repetitions)]
#[derive(Clone, TypeLayout)]
#[repr(transparent)]
pub struct RandRng<G: RandRngCore + RandSeedableRng + Clone + Serialize + DeserializeOwned> {
    inner: G,
}

impl<G: RandRngCore + RandSeedableRng + Clone + Serialize + DeserializeOwned> From<G>
    for RandRng<G>
{
    fn from(inner: G) -> Self {
        Self { inner }
    }
}

impl<G: RandRngCore + RandSeedableRng + Clone + Serialize + DeserializeOwned> RandRng<G> {
    #[must_use]
    pub fn into_inner(self) -> G {
        self.inner
    }
}

impl<G: RandRngCore + RandSeedableRng + Clone + Serialize + DeserializeOwned> fmt::Debug
    for RandRng<G>
{
    default fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        struct InnerRng(&'static str);

        impl fmt::Debug for InnerRng {
            fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
                fmt.write_str(self.0)
            }
        }

        fmt.debug_tuple("RandRng")
            .field(&InnerRng(core::any::type_name::<G>()))
            .finish()
    }
}

impl<G: RandRngCore + RandSeedableRng + Clone + Serialize + DeserializeOwned + fmt::Debug>
    fmt::Debug for RandRng<G>
{
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_tuple("RandRng").field(&self.inner).finish()
    }
}

#[contract_trait]
impl<G: RandRngCore + RandSeedableRng + Clone + Serialize + DeserializeOwned> Backup
    for RandRng<G>
{
    unsafe fn backup_unchecked(&self) -> Self {
        self.clone()
    }
}

impl<G: RandRngCore + RandSeedableRng + Clone + Serialize + DeserializeOwned> Serialize
    for RandRng<G>
{
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.inner.serialize(serializer)
    }
}

impl<'de, G: RandRngCore + RandSeedableRng + Clone + Serialize + DeserializeOwned> Deserialize<'de>
    for RandRng<G>
{
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let inner = G::deserialize(deserializer)?;

        Ok(Self { inner })
    }
}

impl<G: RandRngCore + RandSeedableRng + Clone + Serialize + DeserializeOwned> RngCore
    for RandRng<G>
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
