use core::num::NonZeroUsize;

use alloc::vec::Vec;

use necsim_core::cogs::{
    distribution::{Bernoulli, IndexUsize, Length},
    MathsCore, Rng, SampledDistribution, Samples,
};
use necsim_core_bond::{ClosedUnitF64, NonNegativeF64};

pub mod packed;

#[allow(clippy::module_name_repetitions)]
#[allow(non_snake_case)]
#[derive(Clone)]
pub struct AliasMethodSampler<E: Copy + PartialEq> {
    Us: Vec<ClosedUnitF64>,
    Es: Vec<E>,
    Ks: Vec<E>,
}

impl<E: Copy + PartialEq> AliasMethodSampler<E> {
    #[allow(clippy::no_effect_underscore_binding)]
    #[debug_requires(!event_weights.is_empty(), "event_weights is non-empty")]
    #[debug_ensures(
        ret.Es.iter().eq(old(event_weights).iter().map(|(e, _p)| e)),
        "stores exactly the input events"
    )]
    #[debug_ensures(
        ret.Us.iter().zip(ret.Es.iter()).zip(ret.Ks.iter()).all(|((u, e), k)| {
            let full_bucket = *u == ClosedUnitF64::one();
            !full_bucket || (e == k)
        }),
        "full buckets sample the same event just in case"
    )]
    pub fn new(event_weights: &[(E, NonNegativeF64)]) -> Self {
        #[allow(non_snake_case)]
        let mut Us = Vec::with_capacity(event_weights.len());
        #[allow(non_snake_case)]
        let mut Es = Vec::with_capacity(event_weights.len());
        #[allow(non_snake_case)]
        let mut Ks = Vec::with_capacity(event_weights.len());

        let total_weight: NonNegativeF64 = event_weights.iter().map(|(_e, p)| *p).sum();

        let n = NonNegativeF64::from(event_weights.len());

        for (event, weight) in event_weights {
            Us.push(*weight * n / total_weight);
            Es.push(*event);
            Ks.push(*event);
        }

        let mut overfull_indices: Vec<usize> = (0..Us.len()).filter(|i| Us[*i] > 1.0_f64).collect();
        let mut underfull_indices: Vec<usize> =
            (0..Us.len()).filter(|i| Us[*i] < 1.0_f64).collect();

        while let Some((overfull_index, underfull_index)) =
            pop_overfull_underfull_pair_atomic(&mut overfull_indices, &mut underfull_indices)
        {
            // Safety: Us[overfull_index] > 1.0,
            //         so (Us[overfull_index] - 1.0) > 0.0
            Us[overfull_index] = unsafe {
                NonNegativeF64::new_unchecked(
                    Us[overfull_index].get() + Us[underfull_index].get() - 1.0_f64,
                )
            };
            Ks[underfull_index] = Es[overfull_index];

            #[allow(clippy::comparison_chain)]
            if Us[overfull_index] < 1.0_f64 {
                underfull_indices.push(overfull_index);
            } else if Us[overfull_index] > 1.0_f64 {
                overfull_indices.push(overfull_index);
            }
        }

        // Fix rounding errors for full indices:
        //   M. D. Vose, "A linear algorithm for generating random numbers with a given
        //   distribution", in IEEE Transactions on Software Engineering, vol. 17, no.
        //   9, pp. 972-975, Sept. 1991, doi: 10.1109/32.92917.
        overfull_indices
            .into_iter()
            .for_each(|i| Us[i] = NonNegativeF64::one());
        underfull_indices
            .into_iter()
            .for_each(|i| Us[i] = NonNegativeF64::one());

        // Safety: The bucket weights are now probabilities in [0.0; 1.0]
        #[allow(non_snake_case)]
        let Us = unsafe { core::mem::transmute(Us) };

        Self { Us, Es, Ks }
    }

    #[debug_ensures(self.Es.contains(&ret), "returns one of the weighted events")]
    pub fn sample_event<
        M: MathsCore,
        G: Rng<M> + Samples<M, IndexUsize> + Samples<M, Bernoulli>,
    >(
        &self,
        rng: &mut G,
    ) -> E {
        // Safety: Es is non-empty by the precondition on construction
        let length = unsafe { NonZeroUsize::new_unchecked(self.Es.len()) };

        let i = IndexUsize::sample_with(rng, Length(length)); // index into events

        // Select Es[i] over Ks[i] according to its bucket percentage Us[i]
        if Bernoulli::sample_with(rng, self.Us[i]) {
            self.Es[i]
        } else {
            self.Ks[i]
        }
    }
}

fn pop_overfull_underfull_pair_atomic(
    overfull_indices: &mut Vec<usize>,
    underfull_indices: &mut Vec<usize>,
) -> Option<(usize, usize)> {
    match (overfull_indices.pop(), underfull_indices.pop()) {
        (Some(overfull_index), Some(underfull_index)) => Some((overfull_index, underfull_index)),
        (Some(overfull_index), None) => {
            overfull_indices.push(overfull_index);
            None
        },
        (None, Some(underfull_index)) => {
            underfull_indices.push(underfull_index);
            None
        },
        (None, None) => None,
    }
}
