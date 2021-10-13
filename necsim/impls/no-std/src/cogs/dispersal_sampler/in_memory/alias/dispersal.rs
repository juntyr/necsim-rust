use necsim_core::{
    cogs::{DispersalSampler, F64Core, Habitat, RngCore},
    landscape::Location,
};

use super::InMemoryAliasDispersalSampler;

#[contract_trait]
impl<F: F64Core, H: Habitat<F>, G: RngCore<F>> DispersalSampler<F, H, G>
    for InMemoryAliasDispersalSampler<F, H, G>
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
