use crate::{cogs::Habitat, landscape::IndexedLocation};

#[allow(clippy::inline_always, clippy::inline_fn_without_body)]
#[contract_trait]
pub trait OriginSampler<'h, H: Habitat>:
    core::fmt::Debug + core::iter::Iterator<Item = IndexedLocation>
{
    fn habitat(&self) -> &'h H;

    fn full_upper_bound_size_hint(&self) -> u64;
}
