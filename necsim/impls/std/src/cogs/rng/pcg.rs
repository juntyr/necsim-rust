use std::fmt;

use pcg_rand::{seeds::PcgSeeder, PCGStateInfo, Pcg64};
use rand_core::{RngCore as _, SeedableRng};
use serde::{Deserialize, Serialize};

use necsim_core::cogs::{RngCore, SplittableRng};

#[allow(clippy::module_name_repetitions)]
#[derive(Serialize, Deserialize)]
#[serde(from = "PcgState", into = "PcgState")]
pub struct Pcg {
    inner: Pcg64,
}

impl Clone for Pcg {
    fn clone(&self) -> Self {
        Self {
            inner: Pcg64::restore_state_with_no_verification(self.inner.get_state()),
        }
    }
}

impl fmt::Debug for Pcg {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        let state = self.inner.get_state();

        fmt.debug_struct("Pcg")
            .field("state", &state.state)
            .field("stream", &(state.increment >> 1))
            .finish()
    }
}

impl RngCore for Pcg {
    type Seed = [u8; 16];

    #[must_use]
    #[inline]
    fn from_seed(seed: Self::Seed) -> Self {
        Self {
            inner: Pcg64::from_seed(PcgSeeder::seed_with_stream(
                u128::from_le_bytes(seed),
                0_u128,
            )),
        }
    }

    #[must_use]
    #[inline]
    fn sample_u64(&mut self) -> u64 {
        self.inner.next_u64()
    }
}

impl SplittableRng for Pcg {
    #[allow(clippy::identity_op)]
    fn split(self) -> (Self, Self) {
        let mut left_state = self.inner.get_state();
        left_state.increment = (((left_state.increment >> 1) * 2 + 0) << 1) | 1;

        let mut right_state = self.inner.get_state();
        right_state.increment = (((right_state.increment >> 1) * 2 + 1) << 1) | 1;

        let left = Self {
            inner: Pcg64::restore_state_with_no_verification(left_state),
        };
        let right = Self {
            inner: Pcg64::restore_state_with_no_verification(right_state),
        };

        (left, right)
    }

    fn split_to_stream(self, stream: u64) -> Self {
        let mut state = self.inner.get_state();
        state.increment = (u128::from(stream) << 1) | 1;

        Self {
            inner: Pcg64::restore_state_with_no_verification(state),
        }
    }
}

#[derive(Serialize, Deserialize)]
#[serde(rename = "Pcg")]
#[serde(deny_unknown_fields)]
struct PcgState {
    state: u128,
    increment: u128,
}

impl From<Pcg> for PcgState {
    fn from(rng: Pcg) -> Self {
        let state_info = rng.inner.get_state();

        Self {
            state: state_info.state,
            increment: state_info.increment,
        }
    }
}

impl From<PcgState> for Pcg {
    fn from(state: PcgState) -> Self {
        use pcg_rand::{
            multiplier::{DefaultMultiplier, Multiplier},
            outputmix::{DXsMMixin, OutputMixin},
        };

        let state_info = PCGStateInfo {
            state: state.state,
            increment: state.increment,
            multiplier: DefaultMultiplier::multiplier(),
            internal_width: u128::BITS as usize,
            output_width: u64::BITS as usize,
            output_mixin: <DXsMMixin as OutputMixin<u128, u64>>::SERIALIZER_ID.into(),
        };

        Self {
            inner: Pcg64::restore_state_with_no_verification(state_info),
        }
    }
}
