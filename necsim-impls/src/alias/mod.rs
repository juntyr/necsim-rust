use necsim_core::rng::Rng;

#[allow(clippy::module_name_repetitions)]
#[allow(non_snake_case)]
#[derive(Clone)]
pub struct AliasMethodSampler<E: Copy> {
    Us: Vec<f64>,
    Es: Vec<E>,
    Ks: Vec<E>,
}

impl<E: Copy> AliasMethodSampler<E> {
    #[debug_requires(!event_weights.is_empty(), "event_weights is non-empty")]
    #[debug_requires(
        event_weights.iter().all(|(_e, p)| *p >= 0.0_f64),
        "all event weights are non-negative"
    )]
    // TODO: Can we assert the probability distribution here given rounding errors?
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
            (0..Us.len()).filter(|i| Us[*i] > 1.0_f64).collect();

        while let (Some(overfull_index), Some(underfull_index)) =
            (overfull_indices.pop(), underfull_indices.pop())
        {
            Us[overfull_index] -= 1.0_f64 - Us[underfull_index];
            Ks[underfull_index] = Es[overfull_index];

            if Us[overfull_index] < 1.0_f64 {
                underfull_indices.push(overfull_index);
            } else if Us[overfull_index] > 1.0_f64 {
                overfull_indices.push(overfull_index);
            }
        }

        Self { Us, Es, Ks }
    }

    // TODO: What are our pre- and postconditions here?
    pub fn sample_event(&self, rng: &mut impl Rng) -> E {
        let x = rng.sample_uniform();

        #[allow(
            clippy::cast_precision_loss,
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss
        )]
        let i = (x * (self.Es.len() as f64)).floor() as usize; // index into events

        #[allow(clippy::cast_precision_loss)]
        let y = x * (self.Es.len() as f64) - (i as f64); // U(0,1) to compare against U[i]

        if y < self.Us[i] {
            self.Es[i]
        } else {
            self.Ks[i]
        }
    }
}
