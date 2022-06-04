use core::ops::Index;

use super::{Habitat, LineageReference, MathsCore};
use crate::{
    landscape::{IndexedLocation, Location},
    lineage::{GlobalLineageReference, Lineage},
};

#[allow(clippy::inline_always, clippy::inline_fn_without_body)]
#[contract_trait]
pub trait LineageStore<M: MathsCore, H: Habitat<M>>:
    crate::cogs::Backup + Sized + core::fmt::Debug
{
    type LocalLineageReference: LineageReference<M, H>;

    #[must_use]
    fn with_capacity(habitat: &H, capacity: usize) -> Self;

    #[must_use]
    fn get_lineage_for_local_reference(
        &self,
        reference: Self::LocalLineageReference,
    ) -> Option<&Lineage>;
}

#[allow(clippy::inline_always, clippy::inline_fn_without_body)]
#[allow(clippy::no_effect_underscore_binding)]
#[allow(clippy::module_name_repetitions)]
#[contract_trait]
pub trait LocallyCoherentLineageStore<M: MathsCore, H: Habitat<M>>:
    LineageStore<M, H> + Index<Self::LocalLineageReference, Output = Lineage>
{
    #[must_use]
    #[debug_requires(
        habitat.contains(indexed_location.location()),
        "indexed location is inside habitat"
    )]
    fn get_global_lineage_reference_at_indexed_location(
        &self,
        indexed_location: &IndexedLocation,
        habitat: &H,
    ) -> Option<&GlobalLineageReference>;

    #[debug_requires(
        habitat.contains(lineage.indexed_location.location()),
        "indexed location is inside habitat"
    )]
    #[debug_ensures(self.get_lineage_for_local_reference(
        ret.clone()
    ).is_some(), "lineage was activated")]
    #[debug_ensures(
        self[ret.clone()].indexed_location == old(lineage.indexed_location.clone()),
        "lineage was added to indexed_location"
    )]
    #[debug_ensures(
        self.get_global_lineage_reference_at_indexed_location(
            &old(lineage.indexed_location.clone()), old(habitat)
        ) == Some(&self[ret.clone()].global_reference),
        "lineage is now indexed at indexed_location"
    )]
    fn insert_lineage_locally_coherent(
        &mut self,
        lineage: Lineage,
        habitat: &H,
    ) -> Self::LocalLineageReference;

    #[must_use]
    #[debug_requires(self.get_lineage_for_local_reference(
        reference.clone()
    ).is_some(), "lineage is active")]
    #[debug_ensures(old(habitat).contains(
        ret.indexed_location.location()
    ), "prior location is inside habitat")]
    #[debug_ensures(self.get_lineage_for_local_reference(
        old(reference.clone())
    ).is_none(), "lineage was deactivated")]
    #[debug_ensures(
        ret == old(self[reference.clone()].clone()),
        "returns the individual corresponding to reference"
    )]
    #[debug_ensures(self.get_global_lineage_reference_at_indexed_location(
        &ret.indexed_location, old(habitat)
    ).is_none(), "lineage is no longer indexed at its prior IndexedLocation")]
    fn extract_lineage_locally_coherent(
        &mut self,
        reference: Self::LocalLineageReference,
        habitat: &H,
    ) -> Lineage;
}

#[allow(clippy::inline_always, clippy::inline_fn_without_body)]
#[allow(clippy::no_effect_underscore_binding)]
#[allow(clippy::module_name_repetitions)]
#[contract_trait]
pub trait GloballyCoherentLineageStore<M: MathsCore, H: Habitat<M>>:
    LocallyCoherentLineageStore<M, H>
{
    type LocationIterator<'a>: Iterator<Item = Location>
    where
        M: 'a,
        Self: 'a;

    #[must_use]
    fn iter_active_locations(&self, habitat: &H) -> Self::LocationIterator<'_>;

    #[must_use]
    #[debug_requires(habitat.contains(location), "location is inside habitat")]
    fn get_local_lineage_references_at_location_unordered(
        &self,
        location: &Location,
        habitat: &H,
    ) -> &[Self::LocalLineageReference];

    #[debug_ensures(
        self.get_local_lineage_references_at_location_unordered(
            &old(lineage.indexed_location.location().clone()), old(habitat)
        ).last() == Some(&ret),
        "lineage is now indexed unordered at indexed_location.location()"
    )]
    #[debug_ensures(
        old(self.get_local_lineage_references_at_location_unordered(
            lineage.indexed_location.location(), old(habitat)
        ).len() + 1) == self.get_local_lineage_references_at_location_unordered(
            &old(lineage.indexed_location.location().clone()), old(habitat)
        ).len(),
        "unordered active lineage index at given location has grown by 1"
    )]
    fn insert_lineage_globally_coherent(
        &mut self,
        lineage: Lineage,
        habitat: &H,
    ) -> Self::LocalLineageReference {
        self.insert_lineage_locally_coherent(lineage, habitat)
    }

    #[must_use]
    #[debug_ensures(
        self.get_local_lineage_references_at_location_unordered(
            ret.indexed_location.location(),
            old(habitat),
        ).len() + 1 == old(self.get_local_lineage_references_at_location_unordered(
            self[reference.clone()].indexed_location.location(),
            old(habitat),
        ).len()), "unordered active lineage index at returned location has shrunk by 1")]
    fn extract_lineage_globally_coherent(
        &mut self,
        reference: Self::LocalLineageReference,
        habitat: &H,
    ) -> Lineage {
        self.extract_lineage_locally_coherent(reference, habitat)
    }
}
