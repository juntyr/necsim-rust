use necsim_core::{
    cogs::{Habitat, HabitatPrimeableRng, MathsCore, PrimeableRng, Rng, TurnoverRate},
    landscape::IndexedLocation,
};
use necsim_core_bond::{NonNegativeF64, PositiveF64};

use super::EventTimeSampler;

#[allow(clippy::module_name_repetitions)]
#[derive(Clone, Debug)]
#[cfg_attr(feature = "cuda", derive(rust_cuda::common::LendRustToCuda))]
pub struct ConstEventTimeSampler {
    event_time: PositiveF64,
}

impl ConstEventTimeSampler {
    #[must_use]
    pub fn new(event_time: PositiveF64) -> Self {
        Self { event_time }
    }
}

#[contract_trait]
impl<M: MathsCore, H: Habitat<M>, G: Rng<M, Generator: PrimeableRng>, T: TurnoverRate<M, H>>
    EventTimeSampler<M, H, G, T> for ConstEventTimeSampler
{
    #[inline]
    fn next_event_time_at_indexed_location_weakly_after(
        &self,
        indexed_location: &IndexedLocation,
        _time: NonNegativeF64,
        habitat: &H,
        rng: &mut G,
        _turnover_rate: &T,
    ) -> NonNegativeF64 {
        // Note: Since the constant event time is not controlled by the RNG,
        //        feeding it directly into it should not cause a feedback loop
        rng.generator().prime_with_habitat(
            habitat,
            indexed_location,
            self.event_time.get().to_bits(),
        );

        self.event_time.into()
    }
}
