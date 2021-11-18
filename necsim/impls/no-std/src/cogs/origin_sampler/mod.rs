use necsim_core::{
    cogs::{Habitat, MathsCore},
    lineage::Lineage,
};

pub mod almost_infinite;
pub mod decomposition;
pub mod in_memory;
pub mod non_spatial;
pub mod pre_sampler;
pub mod resuming;
pub mod spatially_implicit;

#[allow(clippy::inline_always, clippy::inline_fn_without_body)]
#[contract_trait]
#[allow(clippy::module_name_repetitions)]
/// `Lineage`s produced by the sampler's iterator must have
/// * unique global references
pub trait UntrustedOriginSampler<'h, M: MathsCore>:
    core::fmt::Debug + core::iter::Iterator<Item = Lineage>
{
    type Habitat: 'h + Habitat<M>;

    fn habitat(&self) -> &'h Self::Habitat;

    fn full_upper_bound_size_hint(&self) -> u64;
}

/// `Lineage`s produced by the sampler's iterator must have
/// * unique global references
/// * unique indexed locations
/// * valid indexed locations (i.e. inside habitable demes)
#[allow(clippy::module_name_repetitions)]
pub unsafe trait TrustedOriginSampler<'h, M: MathsCore>:
    UntrustedOriginSampler<'h, M>
{
}
