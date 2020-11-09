use aes_soft::cipher::generic_array::GenericArray;
use aes_soft::cipher::{BlockCipher, NewBlockCipher};
use aes_soft::Aes128;

#[allow(clippy::module_name_repetitions)]
#[derive(Clone, Debug)]
pub struct AesRng {
    cipher: Aes128,
    state: [u8; 16],
    cached: bool,
}

impl necsim_core::cogs::RngCore for AesRng {
    type Seed = [u8; 16];

    #[must_use]
    #[inline]
    fn from_seed(seed: Self::Seed) -> Self {
        Self {
            cipher: Aes128::new(GenericArray::from_slice(&seed)),
            state: [0_u8; 16],
            cached: false,
        }
    }

    #[must_use]
    #[inline]
    fn sample_u64(&mut self) -> u64 {
        self.cached ^= true;

        if self.cached {
            // one more u64 will be cached
            self.cipher
                .encrypt_block(GenericArray::from_mut_slice(&mut self.state));

            u64::from_le_bytes([
                self.state[0],
                self.state[1],
                self.state[2],
                self.state[3],
                self.state[4],
                self.state[5],
                self.state[6],
                self.state[7],
            ])
        } else {
            // one more u64 was cached
            let rand_u64 = u64::from_le_bytes([
                self.state[8],
                self.state[9],
                self.state[10],
                self.state[11],
                self.state[12],
                self.state[13],
                self.state[14],
                self.state[15],
            ]);

            self.state[9] = self.state[9].wrapping_add(1);

            rand_u64
        }
    }
}

impl necsim_core::cogs::PrimeableRng for AesRng {
    type Prime = [u8; 16];

    fn prime_with(&mut self, prime: Self::Prime) {
        self.state = prime;
        self.cached = false;
    }
}
