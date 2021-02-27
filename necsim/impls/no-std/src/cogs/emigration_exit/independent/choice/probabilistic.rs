use necsim_core::{
    cogs::Habitat,
    intrinsics::{log2, round},
    landscape::IndexedLocation,
};

use super::EmigrationChoice;

#[allow(clippy::module_name_repetitions)]
#[derive(Debug)]
pub struct ProbabilisticEmigrationChoice(());

impl Default for ProbabilisticEmigrationChoice {
    fn default() -> Self {
        Self(())
    }
}

#[contract_trait]
impl<H: Habitat> EmigrationChoice<H> for ProbabilisticEmigrationChoice {
    fn should_lineage_emigrate(
        &self,
        _indexed_location: &IndexedLocation,
        time: f64,
        _habitat: &H,
    ) -> bool {
        if time < 2.0_f64 {
            return true;
        }

        let hash = diffuse(time.to_bits());

        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let time_log2 = round(log2(time)) as u64;

        hash <= (u64::MAX / time_log2)
    }
}

// https://docs.rs/seahash/4.0.1/src/seahash/helper.rs.html#72-89
#[inline]
const fn diffuse(mut x: u64) -> u64 {
    // These are derived from the PCG RNG's round. Thanks to @Veedrac for proposing
    // this. The basic idea is that we use dynamic shifts, which are determined
    // by the input itself. The shift is chosen by the higher bits, which means
    // that changing those flips the lower bits, which scatters upwards because
    // of the multiplication.

    x = x.wrapping_mul(0x6eed_0e9d_a4d9_4a4f);

    let a = x >> 32;
    let b = x >> 60;

    x ^= a >> b;

    x = x.wrapping_mul(0x6eed_0e9d_a4d9_4a4f);

    x
}
