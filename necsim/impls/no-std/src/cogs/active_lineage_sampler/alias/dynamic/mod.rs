use alloc::{vec, vec::Vec};
use core::{
    cmp::Ordering,
    fmt,
    hash::Hash,
    num::{NonZeroU128, NonZeroUsize},
};

use hashbrown::HashMap;

use necsim_core::cogs::{MathsCore, RngCore, RngSampler};
use necsim_core_bond::{NonNegativeF64, PositiveF64};

#[cfg(test)]
mod tests;

#[derive(Clone, Debug, PartialEq, Eq)]
struct RejectionSamplingGroup<E: Eq + Hash> {
    events: Vec<E>,
    weights: Vec<u64>,
    total_weight: u128,
}

#[allow(clippy::module_name_repetitions)]
#[derive(Clone)]
pub struct DynamicAliasMethodSampler<E: Eq + Hash + Clone> {
    exponents: Vec<i16>,
    groups: Vec<RejectionSamplingGroup<E>>,
    lookup: HashMap<E, EventLocation>,
    min_exponent: i16,
    total_weight: u128,
}

impl<E: Eq + Hash + Clone> fmt::Debug for DynamicAliasMethodSampler<E> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_struct("DynamicAliasMethodSampler")
            .field("exponents", &self.exponents)
            .field("total_weight", &self.total_weight().get())
            .finish()
    }
}

impl<E: Eq + Hash> RejectionSamplingGroup<E> {
    fn sample<M: MathsCore, G: RngCore<M>>(&self, rng: &mut G) -> &E {
        loop {
            // Safety: By construction, the group never contains zero elements
            let index =
                rng.sample_index(unsafe { NonZeroUsize::new_unchecked(self.weights.len()) });
            let height = rng.sample_u64() >> 11;

            // 53rd bit of weight is always 1, so sampling chance >= 50%
            if height < self.weights[index] {
                return &self.events[index];
            }
        }
    }

    unsafe fn remove_inplace(
        &mut self,
        index: usize,
        lookup: &mut HashMap<E, EventLocation>,
    ) -> Option<&mut Self> {
        self.events.swap_remove(index);
        let weight = self.weights.swap_remove(index);

        self.total_weight -= u128::from(weight);

        if let Some(event) = self.events.get(index) {
            if let Some(location) = lookup.get_mut(event) {
                location.group_index = index;
            }
        }

        if self.events.is_empty() {
            None
        } else {
            Some(self)
        }
    }

    #[cfg(test)]
    fn remove(mut self, index: usize, lookup: &mut HashMap<E, EventLocation>) -> Option<Self> {
        if unsafe { self.remove_inplace(index, lookup) }.is_some() {
            Some(self)
        } else {
            None
        }
    }

    #[must_use]
    fn add(&mut self, event: E, weight: u64) -> usize {
        self.events.push(event);
        self.weights.push(weight);

        self.total_weight += u128::from(weight);

        self.events.len() - 1
    }

    fn new(event: E, weight: u64) -> Self {
        Self {
            events: vec![event],
            weights: vec![weight],
            total_weight: u128::from(weight),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
struct PositiveF64Decomposed {
    exponent: i16,
    mantissa: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct EventLocation {
    exponent: i16,
    group_index: usize,
}

impl<E: Eq + Hash + Clone> DynamicAliasMethodSampler<E> {
    fn lookup_group_index(&self, exponent: i16) -> Result<usize, usize> {
        for (i, e) in self.exponents.iter().enumerate() {
            match exponent.cmp(e) {
                Ordering::Equal => return Ok(i),
                Ordering::Greater => return Err(i),
                Ordering::Less => continue,
            }
        }

        Err(self.exponents.len())
    }

    fn decompose_weight(weight: PositiveF64) -> PositiveF64Decomposed {
        let bits = weight.get().to_bits();

        #[allow(clippy::cast_possible_truncation)]
        let mut exponent: i16 = ((bits >> 52) & 0x7ff_u64) as i16;

        let mantissa = if exponent == 0 {
            // Ensure that subnormal floats are presented internally as if they were normal
            #[allow(clippy::cast_possible_truncation)]
            let subnormal_exponent = (bits.leading_zeros() as i16) - 12;
            exponent -= subnormal_exponent;

            // weight > 0 && exponent == 0 -> all bits before mantissa are zero
            bits << (bits.leading_zeros() - 11)
        } else {
            // Add the implicit 1.x to the 0.x mantissa
            (bits & 0x000f_ffff_ffff_ffff_u64) | 0x0010_0000_0000_0000_u64
        };

        PositiveF64Decomposed {
            exponent: exponent - 1023,
            mantissa,
        }
    }

    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    fn compose_weight(mut exponent: i16, mut mantissa: u128) -> NonNegativeF64 {
        if mantissa == 0 {
            return NonNegativeF64::zero();
        }

        let mut excess_exponent = 75 - (mantissa.leading_zeros() as i16);

        // Round up if the most significant bit being cut off is 1
        if excess_exponent > 0 && (mantissa & (1_u128 << (excess_exponent - 1))) != 0 {
            mantissa += 1_u128 << excess_exponent;
        }

        excess_exponent = 75 - (mantissa.leading_zeros() as i16);
        exponent += excess_exponent;

        let bits = if exponent >= -1022 {
            // Only keep the 52 bit mantissa for the normal float
            let mantissa_u64 = ((mantissa >> excess_exponent) & 0x000f_ffff_ffff_ffff_u128) as u64;

            (((exponent + 1023) as u64) << 52) | mantissa_u64
        } else {
            // Reconstruct a subnormal float mantissa, its encoded exponent is 0
            let mantissa_u64 = ((mantissa >> (excess_exponent - 1022 - exponent))
                & 0x000f_ffff_ffff_ffff_u128) as u64;

            #[allow(clippy::let_and_return)]
            {
                mantissa_u64
            }
        };

        unsafe { NonNegativeF64::new_unchecked(f64::from_bits(bits)) }
    }

    pub fn add(&mut self, event: E, weight: PositiveF64) {
        self.remove(&event);

        let weight_decomposed = Self::decompose_weight(weight);

        #[allow(clippy::option_if_let_else)]
        let group_index = match self.lookup_group_index(weight_decomposed.exponent) {
            Ok(i) => {
                let group_mut = unsafe { self.groups.get_unchecked_mut(i) };

                group_mut.add(event.clone(), weight_decomposed.mantissa)
            },
            Err(i) => {
                self.exponents.insert(i, weight_decomposed.exponent);
                self.groups.insert(
                    i,
                    RejectionSamplingGroup::new(event.clone(), weight_decomposed.mantissa),
                );

                let old_min_exponent = self.min_exponent;
                self.min_exponent = self.exponents.last().copied().unwrap_or(0_i16);

                if self.min_exponent < old_min_exponent {
                    self.total_weight <<=
                        i32::from(old_min_exponent) - i32::from(self.min_exponent);
                }

                0_usize
            },
        };

        self.total_weight += u128::from(weight_decomposed.mantissa)
            << (i32::from(weight_decomposed.exponent) - i32::from(self.min_exponent));

        self.lookup.insert(
            event,
            EventLocation {
                exponent: weight_decomposed.exponent,
                group_index,
            },
        );
    }

    pub fn remove(&mut self, event: &E) {
        if let Some(location) = self.lookup.remove(event) {
            if let Ok(i) = self.lookup_group_index(location.exponent) {
                let group = unsafe { self.groups.get_unchecked(i) };

                let exponent_shift = i32::from(location.exponent) - i32::from(self.min_exponent);

                self.total_weight = self
                    .total_weight
                    .wrapping_sub(group.total_weight << exponent_shift);

                let group_mut = unsafe { self.groups.get_unchecked_mut(i) };

                if let Some(group) =
                    unsafe { group_mut.remove_inplace(location.group_index, &mut self.lookup) }
                {
                    self.total_weight = self
                        .total_weight
                        .wrapping_add(group.total_weight << exponent_shift);
                } else {
                    self.groups.remove(i);
                    self.exponents.remove(i);

                    let old_min_exponent = self.min_exponent;
                    self.min_exponent = self.exponents.last().copied().unwrap_or(0_i16);

                    if self.min_exponent > old_min_exponent {
                        self.total_weight >>=
                            i32::from(self.min_exponent) - i32::from(old_min_exponent);
                    }
                }
            } else {
                unsafe { core::hint::unreachable_unchecked() }
            }
        }
    }

    #[must_use]
    pub fn new() -> Self {
        Self {
            exponents: Vec::new(),
            groups: Vec::new(),
            lookup: HashMap::new(),
            min_exponent: 0_i16,
            total_weight: 0_u128,
        }
    }

    #[must_use]
    #[allow(dead_code)]
    pub fn with_capacity(capacity: usize) -> Self {
        let capacity_log2_approx = (usize::BITS - capacity.leading_zeros()) as usize;

        Self {
            exponents: Vec::with_capacity(capacity_log2_approx),
            groups: Vec::with_capacity(capacity_log2_approx),
            lookup: HashMap::with_capacity(capacity),
            min_exponent: 0_i16,
            total_weight: 0_u128,
        }
    }

    pub fn sample<M: MathsCore, G: RngCore<M>>(&self, rng: &mut G) -> Option<&E> {
        if let Some(total_weight) = NonZeroU128::new(self.total_weight) {
            let cdf_sample = rng.sample_index_u128(total_weight);

            let mut cdf_acc = 0_u128;

            for (exponent, group) in self.exponents.iter().copied().zip(self.groups.iter()) {
                cdf_acc +=
                    group.total_weight << (i32::from(exponent) - i32::from(self.min_exponent));

                if cdf_sample < cdf_acc {
                    return Some(group.sample(rng));
                }
            }
        }

        None
    }

    #[must_use]
    pub fn total_weight(&self) -> NonNegativeF64 {
        Self::compose_weight(self.min_exponent, self.total_weight)
    }
}

impl<E: Eq + Hash + Clone> Default for DynamicAliasMethodSampler<E> {
    fn default() -> Self {
        Self::new()
    }
}
