use core::cmp::Ordering;

use alloc::vec::Vec;

use necsim_core::{cogs::RngCore, intrinsics::floor};

#[allow(clippy::module_name_repetitions)]
#[allow(non_snake_case)]
#[derive(Clone, Debug)]
#[cfg_attr(feature = "cuda", derive(rust_cuda::rustacuda_core::DeviceCopy))]
#[cfg_attr(feature = "cuda", rustacuda(core = "rust_cuda::rustacuda_core"))]
pub struct AliasMethodSamplerAtom<E: Copy + PartialEq> {
    U: f64,
    E: E,
    K: E,
}

impl<E: Copy + PartialEq> AliasMethodSamplerAtom<E> {
    #[debug_requires(!event_weights.is_empty(), "event_weights is non-empty")]
    #[debug_requires(
        event_weights.iter().all(|(_e, p)| *p >= 0.0_f64),
        "all event weights are non-negative"
    )]
    #[debug_ensures(
        ret.iter().map(|s| &s.E).eq(old(event_weights).iter().map(|(e, _p)| e)),
        "stores exactly the input events"
    )]
    #[debug_ensures(
        ret.iter().all(|s| s.U >= 0.0_f64 && s.U <= 1.0_f64),
        "all bucket probabilities are in U(0, 1)"
    )]
    #[debug_ensures(
        ret.iter().all(|s| {
            let full_bucket = s.U.to_bits() == 1.0_f64.to_bits();
            !full_bucket || (s.E == s.K)
        }),
        "full buckets sample the same event just in case"
    )]
    pub fn create(event_weights: &[(E, f64)]) -> Vec<AliasMethodSamplerAtom<E>> {
        #[allow(non_snake_case)]
        let mut alias_samplers = Vec::with_capacity(event_weights.len());

        let total_weight: f64 = event_weights.iter().map(|(_e, p)| *p).sum();

        #[allow(clippy::cast_precision_loss)]
        let n: f64 = event_weights.len() as f64;

        for (event, weight) in event_weights {
            alias_samplers.push(AliasMethodSamplerAtom {
                U: weight * n / total_weight,
                E: *event,
                K: *event,
            })
        }

        let mut overfull_indices: Vec<usize> = (0..alias_samplers.len())
            .filter(|i| alias_samplers[*i].U > 1.0_f64)
            .collect();
        let mut underfull_indices: Vec<usize> = (0..alias_samplers.len())
            .filter(|i| alias_samplers[*i].U < 1.0_f64)
            .collect();

        while let Some((overfull_index, underfull_index)) =
            pop_overfull_underfull_pair_atomic(&mut overfull_indices, &mut underfull_indices)
        {
            alias_samplers[overfull_index].U -= 1.0_f64 - alias_samplers[underfull_index].U;
            alias_samplers[underfull_index].K = alias_samplers[overfull_index].E;

            match alias_samplers[overfull_index].U.partial_cmp(&1.0_f64) {
                Some(Ordering::Less) => underfull_indices.push(overfull_index),
                Some(Ordering::Greater) => overfull_indices.push(overfull_index),
                _ => (),
            };
        }

        // Fix rounding errors for full indices:
        //   M. D. Vose, "A linear algorithm for generating random numbers with a given
        //   distribution", in IEEE Transactions on Software Engineering, vol. 17, no.
        // 9,   pp. 972-975, Sept. 1991, doi: 10.1109/32.92917.
        overfull_indices
            .into_iter()
            .for_each(|i| alias_samplers[i].U = 1.0_f64);
        underfull_indices
            .into_iter()
            .for_each(|i| alias_samplers[i].U = 1.0_f64);

        alias_samplers
    }

    #[debug_requires(!alias_samplers.is_empty(), "alias_samplers is non-empty")]
    #[debug_ensures(
        old(alias_samplers).iter().map(|s| s.E).any(|e| e == ret),
        "returns one of the weighted events"
    )]
    pub fn sample_event<G: RngCore>(
        alias_samplers: &[AliasMethodSamplerAtom<E>],
        rng: &mut G,
    ) -> E {
        use necsim_core::cogs::RngSampler;

        let x = rng.sample_uniform();

        #[allow(
            clippy::cast_precision_loss,
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss
        )]
        let i = floor(x.get() * (alias_samplers.len() as f64)) as usize; // index into events

        #[allow(clippy::cast_precision_loss)]
        let y = x.get() * (alias_samplers.len() as f64) - (i as f64); // U(0,1) to compare against U[i]

        let sample = &alias_samplers[i];

        if y < sample.U {
            sample.E
        } else {
            sample.K
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
