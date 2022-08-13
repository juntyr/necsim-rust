use necsim_core::{
    cogs::{
        distribution::{Bernoulli, IndexUsize},
        DispersalSampler, Habitat, MathsCore, Rng, Samples,
    },
    landscape::Location,
};

use super::InMemoryAliasDispersalSampler;

#[allow(clippy::trait_duplication_in_bounds)]
#[contract_trait]
impl<M: MathsCore, H: Habitat<M>, G: Rng<M> + Samples<M, IndexUsize> + Samples<M, Bernoulli>>
    DispersalSampler<M, H, G> for InMemoryAliasDispersalSampler<M, H, G>
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
