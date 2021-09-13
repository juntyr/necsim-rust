use std::fmt;

use pcg_rand::{
    multiplier::{DefaultMultiplier, Multiplier},
    outputmix::{DXsMMixin, OutputMixin},
    seeds::PcgSeeder,
    PCGStateInfo, Pcg64,
};
use rand_core::{RngCore as _, SeedableRng};

use necsim_core::cogs::{Backup, RngCore, SplittableRng};

#[allow(clippy::module_name_repetitions)]
pub struct Pcg(Pcg64);

impl Clone for Pcg {
    fn clone(&self) -> Self {
        Self(Pcg64::restore_state_with_no_verification(
            self.0.get_state(),
        ))
    }
}

impl fmt::Debug for Pcg {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_struct("Pcg").finish()
    }
}

#[contract_trait]
impl Backup for Pcg {
    unsafe fn backup_unchecked(&self) -> Self {
        self.clone()
    }
}

impl RngCore for Pcg {
    type Seed = [u8; 16];
    type State = [u8; 32];

    #[must_use]
    fn from_state(inner: Self::State) -> Self {
        let mut state = <[u8; 16]>::default();
        let mut increment = <[u8; 16]>::default();

        state.copy_from_slice(&inner[0..16]);
        increment.copy_from_slice(&inner[16..32]);

        let state_info = PCGStateInfo {
            state: u128::from_le_bytes(state),
            increment: u128::from_le_bytes(increment),
            multiplier: DefaultMultiplier::multiplier(),
            internal_width: u128::BITS as usize,
            output_width: u64::BITS as usize,
            output_mixin: <DXsMMixin as OutputMixin<u128, u64>>::SERIALIZER_ID.into(),
        };

        let pcg = Pcg64::restore_state_with_no_verification(state_info);

        Self(pcg)
    }

    #[must_use]
    fn into_state(self) -> Self::State {
        let state_info = self.0.get_state();

        let mut inner = [0_u8; 32];

        inner[0..16].copy_from_slice(&state_info.state.to_le_bytes());
        inner[16..32].copy_from_slice(&state_info.increment.to_le_bytes());

        inner
    }

    #[must_use]
    #[inline]
    fn from_seed(seed: Self::Seed) -> Self {
        Self(Pcg64::from_seed(PcgSeeder::seed_with_stream(
            u128::from_le_bytes(seed),
            0_u128,
        )))
    }

    #[must_use]
    #[inline]
    fn sample_u64(&mut self) -> u64 {
        self.0.next_u64()
    }
}

impl SplittableRng for Pcg {
    #[allow(clippy::identity_op)]
    fn split(self) -> (Self, Self) {
        let mut left_state = self.0.get_state();
        left_state.increment = (((left_state.increment >> 1) * 2 + 0) << 1) | 1;

        let mut right_state = self.0.get_state();
        right_state.increment = (((right_state.increment >> 1) * 2 + 1) << 1) | 1;

        let left = Self(Pcg64::restore_state_with_no_verification(left_state));
        let right = Self(Pcg64::restore_state_with_no_verification(right_state));

        (left, right)
    }

    fn split_to_stream(self, stream: u64) -> Self {
        let mut state = self.0.get_state();
        state.increment = (u128::from(stream) << 1) | 1;

        Self(Pcg64::restore_state_with_no_verification(state))
    }
}
