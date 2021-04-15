use std::{fmt, io::Write};

use necsim_core::cogs::{Backup, Habitat, PrimeableRng, RngCore};

#[derive(Clone)]
pub struct WriteInterceptingReporter<R: RngCore, W: Write + Clone> {
    inner: R,
    buffer: Box<[u64]>,
    counter: usize,
    writer: W,
}

impl<R: RngCore, W: Write + Clone> fmt::Debug for WriteInterceptingReporter<R, W> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_struct("WriteInterceptingReporter")
            .field("inner", &self.inner)
            .finish()
    }
}

impl<R: RngCore, W: Write + Clone> WriteInterceptingReporter<R, W> {
    pub fn new(rng: R, writer: W, buffer: usize) -> Self {
        Self {
            inner: rng,
            buffer: vec![0_u64; buffer].into_boxed_slice(),
            counter: 0,
            writer,
        }
    }
}

#[contract_trait]
impl<R: RngCore, W: Write + Clone> Backup for WriteInterceptingReporter<R, W> {
    unsafe fn backup_unchecked(&self) -> Self {
        self.clone()
    }
}

impl<R: RngCore, W: Write + Clone> RngCore for WriteInterceptingReporter<R, W> {
    type Seed = R::Seed;

    #[must_use]
    fn from_seed(_seed: Self::Seed) -> Self {
        unimplemented!()
    }

    #[must_use]
    fn sample_u64(&mut self) -> u64 {
        let sample = self.inner.sample_u64();

        self.buffer[self.counter] = sample;

        self.counter += 1;

        if self.counter >= self.buffer.len() {
            self.counter = 0;

            let byte_slice: &[u8] = unsafe {
                std::slice::from_raw_parts(self.buffer.as_ptr().cast(), self.buffer.len() * 8)
            };

            std::mem::drop(self.writer.write_all(byte_slice));
        }

        sample
    }
}

impl<H: Habitat, R: PrimeableRng<H>, W: Write + Clone> PrimeableRng<H>
    for WriteInterceptingReporter<R, W>
{
    fn prime_with(&mut self, location_index: u64, time_index: u64) {
        self.inner.prime_with(location_index, time_index)
    }
}
