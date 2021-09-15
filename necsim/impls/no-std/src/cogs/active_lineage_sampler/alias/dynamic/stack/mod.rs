use alloc::{vec, vec::Vec};
use core::{
    cmp::Ordering,
    fmt,
    hash::Hash,
    num::{NonZeroU128, NonZeroUsize},
};

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
pub struct DynamicAliasMethodStackSampler<E: Eq + Hash + Clone> {
    exponents: Vec<i16>,
    groups: Vec<RejectionSamplingGroup<E>>,
    min_exponent: i16,
    total_weight: u128,

    add_exist: usize,
    add_new: usize,
    remove_remain: usize,
    remove_pop: usize,
}

impl<E: Eq + Hash + Clone> Drop for DynamicAliasMethodStackSampler<E> {
    fn drop(&mut self) {
        info!(
            "\n\n{} {} {} {}\n\n",
            self.add_exist, self.add_new, self.remove_remain, self.remove_pop
        );
    }
}

impl<E: Eq + Hash + Clone> fmt::Debug for DynamicAliasMethodStackSampler<E> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_struct("DynamicAliasMethodStackSampler")
            .field("exponents", &self.exponents)
            .field("total_weight", &self.total_weight().get())
            .finish()
    }
}

impl<E: Eq + Hash> RejectionSamplingGroup<E> {
    unsafe fn sample_pop_inplace<M: MathsCore, G: RngCore<M>>(
        &mut self,
        rng: &mut G,
    ) -> (Option<&mut Self>, E) {
        loop {
            // Safety: By construction, the group never contains zero elements
            let index = rng.sample_index(NonZeroUsize::new_unchecked(self.weights.len()));
            let height = rng.sample_u64() >> 11;

            // 53rd bit of weight is always 1, so sampling chance >= 50%
            if height < self.weights[index] {
                let old_weight = self.weights.swap_remove(index);

                self.total_weight -= u128::from(old_weight);

                let old_event = self.events.swap_remove(index);

                return (
                    if self.events.is_empty() {
                        None
                    } else {
                        Some(self)
                    },
                    old_event,
                );
            }
        }
    }

    // #[cfg(test)]
    // fn sample_pop<G: RngCore>(mut self, rng: &mut G) -> (Option<Self>, E) {
    //     match unsafe { self.sample_pop_inplace(rng) } {
    //         (Some(_), event) => (Some(self), event),
    //         (None, event) => (None, event),
    //     }
    // }

    fn add(&mut self, event: E, weight: u64) {
        self.events.push(event);
        self.weights.push(weight);

        self.total_weight += u128::from(weight);
    }

    fn new(event: E, weight: u64) -> Self {
        Self {
            events: vec![event],
            weights: vec![weight],
            total_weight: u128::from(weight),
        }
    }
}

impl<E: Eq + Hash + Clone> DynamicAliasMethodStackSampler<E> {
    #[must_use]
    pub fn new() -> Self {
        Self {
            exponents: Vec::new(),
            groups: Vec::new(),
            min_exponent: 0_i16,
            total_weight: 0_u128,

            add_exist: 0,
            add_new: 0,
            remove_pop: 0,
            remove_remain: 0,
        }
    }

    #[must_use]
    #[allow(dead_code)]
    pub fn with_capacity(capacity: usize) -> Self {
        let capacity_log2_approx = (usize::BITS - capacity.leading_zeros()) as usize;

        Self {
            exponents: Vec::with_capacity(capacity_log2_approx),
            groups: Vec::with_capacity(capacity_log2_approx),
            min_exponent: 0_i16,
            total_weight: 0_u128,

            add_exist: 0,
            add_new: 0,
            remove_pop: 0,
            remove_remain: 0,
        }
    }

    pub fn sample_pop<M: MathsCore, G: RngCore<M>>(&mut self, rng: &mut G) -> Option<E> {
        if let Some(total_weight) = NonZeroU128::new(self.total_weight) {
            let cdf_sample = rng.sample_index_u128(total_weight);

            let mut cdf_acc = 0_u128;

            for (i, (exponent, group)) in self
                .exponents
                .iter()
                .copied()
                .zip(self.groups.iter_mut())
                .enumerate()
            {
                cdf_acc +=
                    group.total_weight << (i32::from(exponent) - i32::from(self.min_exponent));

                if cdf_sample < cdf_acc {
                    let exponent_shift = i32::from(exponent) - i32::from(self.min_exponent);

                    self.total_weight = self
                        .total_weight
                        .wrapping_sub(group.total_weight << exponent_shift);

                    let (group, sample) = unsafe { group.sample_pop_inplace(rng) };

                    if let Some(group) = group {
                        self.remove_remain += 1;

                        self.total_weight = self
                            .total_weight
                            .wrapping_add(group.total_weight << exponent_shift);
                    } else {
                        self.remove_pop += 1;

                        self.groups.remove(i);
                        self.exponents.remove(i);

                        let old_min_exponent = self.min_exponent;
                        self.min_exponent = self.exponents.last().copied().unwrap_or(0_i16);

                        if self.min_exponent > old_min_exponent {
                            self.total_weight >>=
                                i32::from(self.min_exponent) - i32::from(old_min_exponent);
                        }
                    }

                    return Some(sample);
                }
            }
        }

        None
    }

    pub fn add_push(&mut self, event: E, weight: PositiveF64) {
        let weight_decomposed = super::decompose_weight(weight);

        match self.lookup_group_index(weight_decomposed.exponent) {
            Ok(i) => {
                self.add_exist += 1;

                let group_mut = unsafe { self.groups.get_unchecked_mut(i) };

                group_mut.add(event, weight_decomposed.mantissa);
            },
            Err(i) => {
                self.add_new += 1;

                self.exponents.insert(i, weight_decomposed.exponent);
                self.groups.insert(
                    i,
                    RejectionSamplingGroup::new(event, weight_decomposed.mantissa),
                );

                let old_min_exponent = self.min_exponent;
                self.min_exponent = self.exponents.last().copied().unwrap_or(0_i16);

                if self.min_exponent < old_min_exponent {
                    self.total_weight <<=
                        i32::from(old_min_exponent) - i32::from(self.min_exponent);
                }
            },
        };

        self.total_weight += u128::from(weight_decomposed.mantissa)
            << (i32::from(weight_decomposed.exponent) - i32::from(self.min_exponent));
    }

    #[must_use]
    pub fn total_weight(&self) -> NonNegativeF64 {
        super::compose_weight(self.min_exponent, self.total_weight)
    }

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
}

impl<E: Eq + Hash + Clone> Default for DynamicAliasMethodStackSampler<E> {
    fn default() -> Self {
        Self::new()
    }
}
