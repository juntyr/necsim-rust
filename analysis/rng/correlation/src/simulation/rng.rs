use std::{collections::VecDeque, fmt, marker::PhantomData};

use serde::{Deserialize, Deserializer, Serialize, Serializer};

use necsim_core::cogs::{Backup, MathsCore, PrimeableRng, RngCore};

#[derive(Clone)]
pub struct InterceptingReporter<M: MathsCore, G: RngCore<M>> {
    inner: G,
    buffer: VecDeque<u64>,

    snd_last_reprime: Option<(u64, u64)>,
    last_reprime: Option<(u64, u64)>,
    snd_last_sequence_length: usize,
    cmp_sequence_length: usize,
    sequence_length: usize,

    marker: PhantomData<M>,
}

impl<M: MathsCore, G: RngCore<M>> fmt::Debug for InterceptingReporter<M, G> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_struct(stringify!(InterceptingReporter))
            .field("inner", &self.inner)
            .field("buffer", &self.buffer)
            .finish()
    }
}

impl<M: MathsCore, G: RngCore<M>> InterceptingReporter<M, G> {
    pub fn buffer(&mut self) -> &mut VecDeque<u64> {
        &mut self.buffer
    }
}

#[contract_trait]
impl<M: MathsCore, G: RngCore<M>> Backup for InterceptingReporter<M, G> {
    unsafe fn backup_unchecked(&self) -> Self {
        self.clone()
    }
}

impl<M: MathsCore, G: RngCore<M>> RngCore<M> for InterceptingReporter<M, G> {
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

            marker: PhantomData::<M>,
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

impl<M: MathsCore, G: PrimeableRng<M>> PrimeableRng<M> for InterceptingReporter<M, G> {
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

        self.inner.prime_with(location_index, time_index);
    }
}

impl<M: MathsCore, R: RngCore<M>> Serialize for InterceptingReporter<M, R> {
    fn serialize<S: Serializer>(&self, _serializer: S) -> Result<S::Ok, S::Error> {
        unimplemented!()
    }
}

impl<'de, M: MathsCore, R: RngCore<M>> Deserialize<'de> for InterceptingReporter<M, R> {
    fn deserialize<D: Deserializer<'de>>(_deserializer: D) -> Result<Self, D::Error> {
        unimplemented!()
    }
}
