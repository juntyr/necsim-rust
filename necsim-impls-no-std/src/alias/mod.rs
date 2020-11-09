use necsim_core::cogs::RngCore;

use alloc::vec::Vec;

use necsim_core::intrinsics::floor;

pub mod packed;

#[allow(clippy::module_name_repetitions)]
#[allow(non_snake_case)]
#[derive(Clone)]
pub struct AliasMethodSampler<E: Copy + PartialEq> {
    Us: Vec<f64>,
    Es: Vec<E>,
    Ks: Vec<E>,
}

impl<E: Copy + PartialEq> AliasMethodSampler<E> {
    #[debug_requires(!event_weights.is_empty(), "event_weights is non-empty")]
    #[debug_requires(
        event_weights.iter().all(|(_e, p)| *p >= 0.0_f64),
        "all event weights are non-negative"
    )]
    #[debug_ensures(
        ret.Es.iter().eq(old(event_weights).iter().map(|(e, _p)| e)),
        "stores exactly the input events"
    )]
    #[debug_ensures(
        ret.Us.iter().all(|u| *u >= 0.0_f64 && *u <= 1.0_f64),
        "all bucket probabilities are in U(0, 1)"
    )]
    #[debug_ensures(
        ret.Us.iter().zip(ret.Es.iter()).zip(ret.Ks.iter()).all(|((u, e), k)| {
            #[allow(clippy::float_cmp)]
            let full_bucket = *u == 1.0_f64;
            !full_bucket || (e == k)
        }),
        "full buckets sample the same event just in case"
    )]
    pub fn new(event_weights: &[(E, f64)]) -> Self {
        #[allow(non_snake_case)]
        let mut Us = Vec::with_capacity(event_weights.len());
        #[allow(non_snake_case)]
        let mut Es = Vec::with_capacity(event_weights.len());
        #[allow(non_snake_case)]
        let mut Ks = Vec::with_capacity(event_weights.len());

        let total_weight: f64 = event_weights.iter().map(|(_e, p)| *p).sum();

        #[allow(clippy::cast_precision_loss)]
        let n: f64 = event_weights.len() as f64;

        for (event, weight) in event_weights {
            Us.push(weight * n / total_weight);
            Es.push(*event);
            Ks.push(*event);
        }

        let mut overfull_indices: Vec<usize> = (0..Us.len()).filter(|i| Us[*i] > 1.0_f64).collect();
        let mut underfull_indices: Vec<usize> =
            (0..Us.len()).filter(|i| Us[*i] < 1.0_f64).collect();

        while let Some((overfull_index, underfull_index)) =
            pop_overfull_underfull_pair_atomic(&mut overfull_indices, &mut underfull_indices)
        {
            Us[overfull_index] -= 1.0_f64 - Us[underfull_index];
            Ks[underfull_index] = Es[overfull_index];

            if Us[overfull_index] < 1.0_f64 {
                underfull_indices.push(overfull_index);
            } else if Us[overfull_index] > 1.0_f64 {
                overfull_indices.push(overfull_index);
            }
        }

        // Fix rounding errors for full indices:
        //   M. D. Vose, "A linear algorithm for generating random numbers with a given
        //   distribution", in IEEE Transactions on Software Engineering, vol. 17, no. 9,
        //   pp. 972-975, Sept. 1991, doi: 10.1109/32.92917.
        overfull_indices.into_iter().for_each(|i| Us[i] = 1.0_f64);
        underfull_indices.into_iter().for_each(|i| Us[i] = 1.0_f64);

        Self { Us, Es, Ks }
    }

    #[debug_ensures(self.Es.contains(&ret), "returns one of the weighted events")]
    pub fn sample_event<G: RngCore>(&self, rng: &mut G) -> E {
        use necsim_core::cogs::RngSampler;

        let x = rng.sample_uniform();

        #[allow(
            clippy::cast_precision_loss,
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss
        )]
        let i = floor(x * (self.Es.len() as f64)) as usize; // index into events

        #[allow(clippy::cast_precision_loss)]
        let y = x * (self.Es.len() as f64) - (i as f64); // U(0,1) to compare against U[i]

        if y < self.Us[i] {
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
        }
        (None, Some(underfull_index)) => {
            underfull_indices.push(underfull_index);
            None
        }
        (None, None) => None,
    }
}
