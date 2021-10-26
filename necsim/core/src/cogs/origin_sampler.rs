use crate::{
    cogs::{Habitat, MathsCore},
    lineage::Lineage,
};

#[allow(clippy::inline_always, clippy::inline_fn_without_body)]
#[contract_trait]
pub trait OriginSampler<'h, M: MathsCore>:
    core::fmt::Debug + core::iter::Iterator<Item = Lineage>
{
    type Habitat: 'h + Habitat<M>;

    fn habitat(&self) -> &'h Self::Habitat;

    fn full_upper_bound_size_hint(&self) -> u64;
}
