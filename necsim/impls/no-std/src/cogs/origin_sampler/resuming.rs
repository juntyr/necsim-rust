use core::{fmt, iter::ExactSizeIterator};

use necsim_core::{
    cogs::{Habitat, MathsCore, OriginSampler},
    lineage::Lineage,
};

use crate::cogs::origin_sampler::pre_sampler::OriginPreSampler;

#[allow(clippy::module_name_repetitions)]
pub struct ResumingOriginSampler<
    'h,
    'o,
    M: MathsCore,
    H: Habitat<M>,
    L: ExactSizeIterator<Item = Lineage>,
    I: Iterator<Item = u64>,
    O: FnMut(Lineage),
> {
    lineage_iterator: L,
    oob_lineage_generator: &'o mut O,
    pre_sampler: OriginPreSampler<M, I>,
    last_index: u64,
    habitat: &'h H,
}

impl<
        'h,
        'o,
        M: MathsCore,
        H: Habitat<M>,
        L: ExactSizeIterator<Item = Lineage>,
        I: Iterator<Item = u64>,
        O: FnMut(Lineage),
    > fmt::Debug for ResumingOriginSampler<'h, 'o, M, H, L, I, O>
{
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_struct(stringify!(ResumingOriginSampler))
            .field("pre_sampler", &self.pre_sampler)
            .field("last_index", &self.last_index)
            .field("habitat", &self.habitat)
            .finish()
    }
}

impl<
        'h,
        'o,
        M: MathsCore,
        H: Habitat<M>,
        L: ExactSizeIterator<Item = Lineage>,
        I: Iterator<Item = u64>,
        O: FnMut(Lineage),
    > ResumingOriginSampler<'h, 'o, M, H, L, I, O>
{
    #[must_use]
    pub fn new(
        lineage_iterator: L,
        pre_sampler: OriginPreSampler<M, I>,
        habitat: &'h H,
        oob_lineage_generator: &'o mut O,
    ) -> Self {
        // TODO: the output of this sampler could be further sampled, so the
        //       oob generator might contain out-of-partition items
        Self {
            lineage_iterator,
            oob_lineage_generator,
            pre_sampler,
            last_index: 0_u64,
            habitat,
        }
    }
}

#[contract_trait]
impl<
        'h,
        'o,
        M: MathsCore,
        H: Habitat<M>,
        L: ExactSizeIterator<Item = Lineage>,
        I: Iterator<Item = u64>,
        O: FnMut(Lineage),
    > OriginSampler<'h, M> for ResumingOriginSampler<'h, 'o, M, H, L, I, O>
{
    type Habitat = H;

    fn habitat(&self) -> &'h Self::Habitat {
        self.habitat
    }

    fn full_upper_bound_size_hint(&self) -> u64 {
        #[allow(
            clippy::cast_precision_loss,
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss
        )]
        let upper_bound_size_hint = M::ceil(
            (self.lineage_iterator.len() as f64) * self.pre_sampler.get_sample_proportion(),
        ) as u64;

        upper_bound_size_hint
    }
}

impl<
        'h,
        'o,
        M: MathsCore,
        H: Habitat<M>,
        L: ExactSizeIterator<Item = Lineage>,
        I: Iterator<Item = u64>,
        O: FnMut(Lineage),
    > Iterator for ResumingOriginSampler<'h, 'o, M, H, L, I, O>
{
    type Item = Lineage;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let next_index = self.pre_sampler.next()?;
            let index_difference = next_index - self.last_index;
            self.last_index = next_index + 1;

            for _ in 0..index_difference {
                self.lineage_iterator.next()?;
            }

            let lineage = self.lineage_iterator.next()?;

            #[allow(clippy::redundant_else)]
            if self.habitat.contains(lineage.indexed_location.location())
                && lineage.indexed_location.index()
                    < self
                        .habitat
                        .get_habitat_at_location(lineage.indexed_location.location())
            {
                // if the lineage is inside the habitat, return it
                return Some(lineage);
            } else {
                // otherwise push the lineage to the oob generator
                (self.oob_lineage_generator)(lineage);
            }
        }
    }
}
