use std::fmt;

use pcg_rand::{seeds::PcgSeeder, Pcg64};
use rand::{RngCore as _, SeedableRng};

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
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
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
