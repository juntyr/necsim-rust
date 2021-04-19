use std::{collections::VecDeque, fmt};

use necsim_core::cogs::{Backup, PrimeableRng, RngCore};

#[derive(Clone)]
pub struct InterceptingReporter<G: RngCore> {
    inner: G,
    buffer: VecDeque<u64>,

    snd_last_reprime: Option<(u64, u64)>,
    last_reprime: Option<(u64, u64)>,
    snd_last_sequence_length: usize,
    cmp_sequence_length: usize,
    sequence_length: usize,
}

impl<G: RngCore> fmt::Debug for InterceptingReporter<G> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_struct("InterceptingReporter")
            .field("inner", &self.inner)
            .field("buffer", &self.buffer)
            .finish()
    }
}

impl<G: RngCore> InterceptingReporter<G> {
    pub fn buffer(&mut self) -> &mut VecDeque<u64> {
        &mut self.buffer
    }
}

#[contract_trait]
impl<G: RngCore> Backup for InterceptingReporter<G> {
    unsafe fn backup_unchecked(&self) -> Self {
        self.clone()
    }
}

impl<G: RngCore> RngCore for InterceptingReporter<G> {
    type Seed = G::Seed;

    #[must_use]
    fn from_seed(seed: Self::Seed) -> Self {
        Self {
            inner: G::from_seed(seed),
            buffer: VecDeque::new(),

            snd_last_reprime: None,
            last_reprime: None,
            snd_last_sequence_length: 0,
            cmp_sequence_length: 0,
            sequence_length: 0,
        }
    }

    #[must_use]
    fn sample_u64(&mut self) -> u64 {
        let sample = self.inner.sample_u64();

        self.sequence_length += 1;

        if self.sequence_length > self.cmp_sequence_length {
            self.buffer.push_back(sample);
        }

        sample
    }
}

impl<G: PrimeableRng> PrimeableRng for InterceptingReporter<G> {
    fn prime_with(&mut self, location_index: u64, time_index: u64) {
        if Some((location_index, time_index)) == self.snd_last_reprime {
            self.cmp_sequence_length = self.snd_last_sequence_length;
        } else {
            self.cmp_sequence_length = 0;
        }

        self.snd_last_reprime = self.last_reprime;
        self.snd_last_sequence_length = self.sequence_length;

        self.last_reprime = Some((location_index, time_index));

        self.sequence_length = 0;

        self.inner.prime_with(location_index, time_index)
    }
}
