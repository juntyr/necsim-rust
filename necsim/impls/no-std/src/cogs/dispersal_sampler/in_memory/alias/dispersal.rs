use necsim_core::{
    cogs::{
        distribution::{Bernoulli, IndexUsize},
        DispersalSampler, DistributionSampler, Habitat, MathsCore, Rng,
    },
    landscape::Location,
};

use super::InMemoryAliasDispersalSampler;

#[contract_trait]
impl<M: MathsCore, H: Habitat<M>, G: Rng<M>> DispersalSampler<M, H, G>
    for InMemoryAliasDispersalSampler<M, H, G>
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
        let alias_dispersal_at_location = self.alias_dispersal[(
            (location.y() - habitat.get_extent().y()) as usize,
            (location.x() - habitat.get_extent().x()) as usize,
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
}
