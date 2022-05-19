use necsim_core::{
    cogs::{
        distribution::{Bernoulli, IndexUsize},
        DispersalSampler, DistributionSampler, Habitat, MathsCore, Rng, SampledDistribution,
        SeparableDispersalSampler,
    },
    landscape::Location,
};
use necsim_core_bond::ClosedUnitF64;

use super::InMemorySeparableAliasDispersalSampler;

#[contract_trait]
impl<M: MathsCore, H: Habitat<M>, G: Rng<M>> DispersalSampler<M, H, G>
    for InMemorySeparableAliasDispersalSampler<M, H, G>
where
    G::Sampler: DistributionSampler<M, G::Generator, G::Sampler, IndexUsize>
        + DistributionSampler<M, G::Generator, G::Sampler, Bernoulli>,
{
    #[must_use]
    fn sample_dispersal_from_location(
        &self,
        location: &Location,
        habitat: &H,
        rng: &mut G,
    ) -> Location {
        let self_dispersal_at_location =
            self.get_self_dispersal_probability_at_location(location, habitat);

        if self_dispersal_at_location >= 1.0_f64 {
            return location.clone();
        }

        if self_dispersal_at_location > 0.0_f64
            && Bernoulli::sample_with(rng, self_dispersal_at_location)
        {
            return location.clone();
        }

        self.sample_non_self_dispersal_from_location(location, habitat, rng)
    }
}

#[contract_trait]
impl<M: MathsCore, H: Habitat<M>, G: Rng<M>> SeparableDispersalSampler<M, H, G>
    for InMemorySeparableAliasDispersalSampler<M, H, G>
where
    G::Sampler: DistributionSampler<M, G::Generator, G::Sampler, IndexUsize>
        + DistributionSampler<M, G::Generator, G::Sampler, Bernoulli>,
{
    #[must_use]
    fn sample_non_self_dispersal_from_location(
        &self,
        location: &Location,
        habitat: &H,
        rng: &mut G,
    ) -> Location {
        let alias_dispersal_at_location = self.alias_dispersal[(
            location.y().wrapping_sub(habitat.get_extent().y()) as usize,
            location.x().wrapping_sub(habitat.get_extent().x()) as usize,
        )]
            .as_ref()
            .expect("habitat dispersal origin must disperse somewhere");

        let dispersal_target_index = alias_dispersal_at_location.sample_event(rng);

        #[allow(clippy::cast_possible_truncation)]
        Location::new(
            habitat.get_extent().x().wrapping_add(
                (dispersal_target_index % usize::from(habitat.get_extent().width())) as u32,
            ),
            habitat.get_extent().y().wrapping_add(
                (dispersal_target_index / usize::from(habitat.get_extent().width())) as u32,
            ),
        )
    }

    #[must_use]
    #[debug_requires(habitat.get_extent().contains(location), "location is inside habitat extent")]
    fn get_self_dispersal_probability_at_location(
        &self,
        location: &Location,
        habitat: &H,
    ) -> ClosedUnitF64 {
        self.self_dispersal[(
            location.y().wrapping_sub(habitat.get_extent().y()) as usize,
            location.x().wrapping_sub(habitat.get_extent().x()) as usize,
        )]
    }
}
