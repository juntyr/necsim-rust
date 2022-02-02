use std::{fmt, marker::PhantomData};

use serde::{Deserialize, Deserializer, Serialize, Serializer};

use necsim_core::cogs::{MathsCore, RngCore};

mod base32;

use self::base32::Base32String;

#[allow(dead_code)]
pub struct Base32RngState<M: MathsCore, G: RngCore<M>> {
    rng: G,
    marker: PhantomData<M>,
}

impl<M: MathsCore, G: RngCore<M>> From<G> for Base32RngState<M, G> {
    fn from(rng: G) -> Self {
        Self {
            rng,
            marker: PhantomData::<M>,
        }
    }
}

impl<M: MathsCore, G: RngCore<M>> Base32RngState<M, G> {
    #[must_use]
    #[allow(dead_code)]
    pub fn into(self) -> G {
        self.rng
    }
}

impl<M: MathsCore, G: RngCore<M>> fmt::Debug for Base32RngState<M, G> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match bincode::Options::serialize(bincode::options(), &self.rng) {
            Ok(state) => Base32String::new(&state).fmt(fmt),
            Err(_) => fmt.write_str("InvalidRngState"),
        }
    }
}

impl<M: MathsCore, G: RngCore<M>> Serialize for Base32RngState<M, G> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let state = bincode::Options::serialize(bincode::options(), &self.rng)
            .map_err(serde::ser::Error::custom)?;

        Base32String::new(&state).serialize(serializer)
    }
}

impl<'de, M: MathsCore, G: RngCore<M>> Deserialize<'de> for Base32RngState<M, G> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let state = Base32String::deserialize(deserializer)?;

        let rng = bincode::Options::deserialize(bincode::options(), &state)
            .map_err(|_| serde::de::Error::custom(format!("invalid RNG state {}", state)))?;

        Ok(Self {
            rng,
            marker: PhantomData::<M>,
        })
    }
}

#[derive(Debug, Serialize)]
#[serde(bound = "")]
pub enum Rng<M: MathsCore, G: RngCore<M>> {
    Seed(u64),
    Sponge(Base32String),
    State(Base32RngState<M, G>),
}

impl<'de, M: MathsCore, G: RngCore<M>> Deserialize<'de> for Rng<M, G> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let raw = RngRaw::<M, G>::deserialize(deserializer)?;

        let rng = match raw {
            RngRaw::Entropy => {
                let mut entropy = G::Seed::default();

                getrandom::getrandom(entropy.as_mut()).map_err(serde::de::Error::custom)?;

                Self::Sponge(Base32String::new(entropy.as_mut()))
            },
            RngRaw::Seed(seed) => Self::Seed(seed),
            RngRaw::Sponge(sponge) => Self::Sponge(sponge),
            RngRaw::State(state) => Self::State(state),
            RngRaw::StateElseSponge(state) => {
                match bincode::Options::deserialize(bincode::options(), &state) {
                    Ok(rng) => Self::State(Base32RngState {
                        rng,
                        marker: PhantomData::<M>,
                    }),
                    Err(_) => Self::Sponge(state),
                }
            },
        };

        Ok(rng)
    }
}

#[derive(Debug, Deserialize)]
#[serde(bound = "")]
#[serde(rename = "Rng")]
enum RngRaw<M: MathsCore, G: RngCore<M>> {
    Entropy,
    Seed(u64),
    Sponge(Base32String),
    State(Base32RngState<M, G>),
    StateElseSponge(Base32String),
}
