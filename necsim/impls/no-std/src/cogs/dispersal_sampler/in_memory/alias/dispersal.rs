use necsim_core::{
    cogs::{DispersalSampler, Habitat, MathsCore, RngCore},
    landscape::Location,
};

use super::InMemoryAliasDispersalSampler;

#[contract_trait]
impl<M: MathsCore, H: Habitat<M>, G: RngCore<M>> DispersalSampler<M, H, G>
    for InMemoryAliasDispersalSampler<M, H, G>
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
            (dispersal_target_index % (habitat.get_extent().width() as usize)) as u32
                + habitat.get_extent().x(),
            (dispersal_target_index / (habitat.get_extent().width() as usize)) as u32
                + habitat.get_extent().y(),
        )
    }
}
