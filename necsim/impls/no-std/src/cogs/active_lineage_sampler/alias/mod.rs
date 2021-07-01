use alloc::{vec, vec::Vec};
use core::{
    hash::Hash,
    num::{NonZeroU128, NonZeroUsize},
};

use hashbrown::HashMap;

use necsim_core::cogs::{MathsCore, RngCore, RngSampler};
use necsim_core_bond::PositiveF64;

struct RejectionSamplingGroup<E: Eq + Hash + Copy> {
    events: Vec<E>,
    weights: Vec<u64>,
    total_weight: u128,
}

pub struct DynamicAliasMethodSampler<E: Eq + Hash + Copy> {
    exponents: Vec<i16>,
    groups: Vec<RejectionSamplingGroup<E>>,
    lookup: HashMap<E, EventLocation>,
    min_exponent: i16,
    total_weight: u128,
}

impl<E: Eq + Hash + Copy> RejectionSamplingGroup<E> {
    fn sample<M: MathsCore, G: RngCore<M>>(&self, rng: &mut G) -> &E {
        loop {
            // Safety: By construction, the group never contains zero elements
            let index =
                rng.sample_index(unsafe { NonZeroUsize::new_unchecked(self.weights.len()) });
            let height = rng.sample_u64() >> 11;

            // >= 50% chance we hit the event bar in any case
            if height >= (0x1_u64 << 52) {
                return &self.events[index];
            }

            if height < self.weights[index] {
                return &self.events[index];
            }
        }
    }

    fn remove(mut self, index: usize, lookup: &mut HashMap<E, EventLocation>) -> Option<Self> {
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

struct PositiveF64Decomposed {
    exponent: i16,
    mantissa: u64,
}

struct EventLocation {
    exponent: i16,
    group_index: usize,
}

impl<E: Eq + Hash + Copy> DynamicAliasMethodSampler<E> {
    fn lookup_mut(&mut self, exponent: i16) -> Option<&mut RejectionSamplingGroup<E>> {
        self.exponents
            .iter()
            .position(|e| *e == exponent)
            .and_then(move |i| self.groups.get_mut(i))
    }

    fn lookup_take(&mut self, exponent: i16) -> Option<RejectionSamplingGroup<E>> {
        self.exponents
            .iter()
            .position(|e| *e == exponent)
            .map(|index| {
                self.exponents.swap_remove(index);
                self.groups.swap_remove(index)
            })
    }

    fn decompose_weight(weight: PositiveF64) -> PositiveF64Decomposed {
        let bits = weight.get().to_bits();

        #[allow(clippy::cast_possible_truncation)]
        let exponent: i16 = ((bits >> 52) & 0x7ff_u64) as i16;

        let mantissa = if exponent == 0 {
            (bits & 0x000f_ffff_ffff_ffff_u64) << 1
        } else {
            (bits & 0x000f_ffff_ffff_ffff_u64) | 0x0010_0000_0000_0000_u64
        };

        PositiveF64Decomposed {
            exponent: exponent - (1023 + 52),
            mantissa,
        }
    }

    pub fn add(&mut self, event: E, weight: PositiveF64) {
        self.remove(&event);

        let weight_decomposed = Self::decompose_weight(weight);

        #[allow(clippy::option_if_let_else)]
        let group_index = if let Some(group) = self.lookup_mut(weight_decomposed.exponent) {
            group.add(event, weight_decomposed.mantissa)
        } else {
            self.exponents.push(weight_decomposed.exponent);
            self.groups.push(RejectionSamplingGroup::new(
                event,
                weight_decomposed.mantissa,
            ));

            let old_min_exponent = self.min_exponent;
            self.min_exponent = self.exponents.iter().copied().min().unwrap_or(0_i16);

            if self.min_exponent < old_min_exponent {
                self.total_weight <<= i32::from(old_min_exponent) - i32::from(self.min_exponent);
            }

            0_usize
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
            if let Some(group) = self.lookup_take(location.exponent) {
                let exponent_shift = i32::from(location.exponent) - i32::from(self.min_exponent);

                self.total_weight = self
                    .total_weight
                    .wrapping_sub(group.total_weight << exponent_shift);

                if let Some(group) = group.remove(location.group_index, &mut self.lookup) {
                    self.total_weight = self
                        .total_weight
                        .wrapping_add(group.total_weight << exponent_shift);

                    self.exponents.push(location.exponent);
                    self.groups.push(group);
                } else {
                    let old_min_exponent = self.min_exponent;
                    self.min_exponent = self.exponents.iter().copied().min().unwrap_or(0_i16);

                    if self.min_exponent > old_min_exponent {
                        self.total_weight >>=
                            i32::from(self.min_exponent) - i32::from(old_min_exponent);
                    }
                }
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
}

impl<E: Eq + Hash + Copy> Default for DynamicAliasMethodSampler<E> {
    fn default() -> Self {
        Self::new()
    }
}
