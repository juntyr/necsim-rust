use crate::{cogs::Habitat, landscape::IndexedLocation};

#[allow(clippy::inline_always, clippy::inline_fn_without_body)]
#[contract_trait]
pub trait OriginSampler<'h>:
    core::fmt::Debug + core::iter::Iterator<Item = IndexedLocation>
{
    type Habitat: 'h + Habitat;

    fn habitat(&self) -> &'h Self::Habitat;

    fn full_upper_bound_size_hint(&self) -> u64;
}
