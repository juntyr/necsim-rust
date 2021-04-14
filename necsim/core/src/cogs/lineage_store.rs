use core::ops::Index;

use super::{Habitat, LineageReference, OriginSampler};
use crate::{
    landscape::{IndexedLocation, Location},
    lineage::{GlobalLineageReference, Lineage},
};

#[allow(clippy::inline_always, clippy::inline_fn_without_body)]
#[contract_trait]
pub trait LineageStore<H: Habitat, R: LineageReference<H>>:
    crate::cogs::Backup + Sized + core::fmt::Debug
{
    type LineageReferenceIterator<'a>: Iterator<Item = R>;

    #[must_use]
    fn from_origin_sampler<'h, O: OriginSampler<'h, Habitat = H>>(origin_sampler: O) -> Self
    where
        H: 'h;

    #[must_use]
    fn get_number_total_lineages(&self) -> usize;

    #[must_use]
    fn iter_local_lineage_references(&self) -> Self::LineageReferenceIterator<'_>;

    #[must_use]
    fn get(&self, reference: R) -> Option<&Lineage>;
}

#[allow(clippy::inline_always, clippy::inline_fn_without_body)]
#[allow(clippy::module_name_repetitions)]
#[contract_trait]
pub trait LocallyCoherentLineageStore<H: Habitat, R: LineageReference<H>>:
    LineageStore<H, R> + Index<R, Output = Lineage>
{
    #[must_use]
    #[debug_requires(
        habitat.contains(indexed_location.location()),
        "indexed location is inside habitat"
    )]
    fn get_active_global_lineage_reference_at_indexed_location(
        &self,
        indexed_location: &IndexedLocation,
        habitat: &H,
    ) -> Option<&GlobalLineageReference>;

    #[debug_requires(
        habitat.contains(indexed_location.location()),
        "indexed location is inside habitat"
    )]
    #[debug_requires(self.get(reference.clone()).is_some(), "lineage reference is valid")]
    #[debug_requires(!self[reference.clone()].is_active(), "lineage is inactive")]
    #[debug_ensures(self[old(reference.clone())].is_active(), "lineage was activated")]
    #[debug_ensures(
        self[old(reference.clone())].indexed_location() == Some(&old(indexed_location.clone())),
        "lineage was added to indexed_location"
    )]
    #[debug_ensures(
        self.get_active_global_lineage_reference_at_indexed_location(
            &old(indexed_location.clone()), old(habitat)
        ) == Some(self[old(reference.clone())].global_reference()),
        "lineage is now indexed at indexed_location"
    )]
    fn insert_lineage_to_indexed_location_locally_coherent(
        &mut self,
        reference: R,
        indexed_location: IndexedLocation,
        habitat: &H,
    );

    #[must_use]
    #[debug_requires(self.get(reference.clone()).is_some(), "lineage reference is valid")]
    #[debug_requires(self[reference.clone()].is_active(), "lineage is active")]
    #[debug_ensures(old(habitat).contains(ret.0.location()), "prior location is inside habitat")]
    #[debug_ensures(!self[old(reference.clone())].is_active(), "lineage was deactivated")]
    #[debug_ensures(
        ret.0 == old(self[reference.clone()].indexed_location().unwrap().clone()),
        "returns the individual's prior IndexedLocation"
    )]
    #[debug_ensures(
        ret.1.to_bits() == old(self[reference.clone()].last_event_time().to_bits()),
        "returns the individual's prior last event time"
    )]
    #[debug_ensures(self.get_active_global_lineage_reference_at_indexed_location(
        &ret.0, old(habitat)
    ).is_none(), "lineage is no longer indexed at its prior IndexedLocation")]
    #[debug_ensures(
        self[old(reference.clone())].last_event_time().to_bits() == old(event_time.to_bits()),
        "updates the time of the last event of the lineage reference"
    )]
    fn extract_lineage_from_its_location_locally_coherent(
        &mut self,
        reference: R,
        event_time: f64,
        habitat: &H,
    ) -> (IndexedLocation, f64);

    #[debug_requires(
        self.get(local_lineage_reference.clone()).is_some(),
        "lineage reference is valid"
    )]
    #[debug_requires(
        !self[local_lineage_reference.clone()].is_active(),
        "lineage is inactive"
    )]
    #[debug_ensures(
        self.get(old(local_lineage_reference.clone())).is_none(),
        "lineage was removed"
    )]
    #[debug_ensures(
        ret == old(self[local_lineage_reference.clone()].global_reference().clone()),
        "returns the individual's GlobalLineageReference"
    )]
    fn emigrate(&mut self, local_lineage_reference: R) -> GlobalLineageReference;

    #[must_use]
    #[debug_requires(
        habitat.contains(indexed_location.location()),
        "indexed location is inside habitat"
    )]
    #[debug_ensures(self[ret.clone()].is_active(), "lineage was activated")]
    #[debug_ensures(
        self[ret.clone()].indexed_location() == Some(&old(indexed_location.clone())),
        "lineage was added to indexed_location"
    )]
    #[debug_ensures(
        self.get_active_global_lineage_reference_at_indexed_location(
            &old(indexed_location.clone()), old(habitat)
        ) == Some(self[ret.clone()].global_reference()),
        "lineage is now indexed at indexed_location"
    )]
    fn immigrate_locally_coherent(
        &mut self,
        habitat: &H,
        global_reference: GlobalLineageReference,
        indexed_location: IndexedLocation,
        time_of_emigration: f64,
    ) -> R;
}

#[allow(clippy::inline_always, clippy::inline_fn_without_body)]
#[allow(clippy::module_name_repetitions)]
#[contract_trait]
pub trait GloballyCoherentLineageStore<H: Habitat, R: LineageReference<H>>:
    LocallyCoherentLineageStore<H, R>
{
    type LocationIterator<'a>: Iterator<Item = Location>;

    #[must_use]
    fn iter_active_locations(&self, habitat: &H) -> Self::LocationIterator<'_>;

    #[must_use]
    #[debug_requires(habitat.contains(location), "location is inside habitat")]
    fn get_active_local_lineage_references_at_location_unordered(
        &self,
        location: &Location,
        habitat: &H,
    ) -> &[R];

    #[debug_ensures(
        self.get_active_local_lineage_references_at_location_unordered(
            &old(indexed_location.location().clone()), old(habitat)
        ).last() == Some(&old(reference.clone())),
        "lineage is now indexed unordered at indexed_location.location()"
    )]
    #[debug_ensures(
        old(self.get_active_local_lineage_references_at_location_unordered(
            indexed_location.location(), old(habitat)
        ).len() + 1) == self.get_active_local_lineage_references_at_location_unordered(
            &old(indexed_location.location().clone()), old(habitat)
        ).len(),
        "unordered active lineage index at given location has grown by 1"
    )]
    fn insert_lineage_to_indexed_location_globally_coherent(
        &mut self,
        reference: R,
        indexed_location: IndexedLocation,
        habitat: &H,
    ) {
        self.insert_lineage_to_indexed_location_locally_coherent(
            reference,
            indexed_location,
            habitat,
        )
    }

    #[must_use]
    #[debug_ensures(
        self.get_active_local_lineage_references_at_location_unordered(
            &ret.0.location(),
            old(habitat),
        ).len() + 1 == old(self.get_active_local_lineage_references_at_location_unordered(
            self[reference.clone()].indexed_location().unwrap().location(),
            old(habitat),
        ).len()), "unordered active lineage index at returned location has shrunk by 1")]
    fn extract_lineage_from_its_location_globally_coherent(
        &mut self,
        reference: R,
        event_time: f64,
        habitat: &H,
    ) -> (IndexedLocation, f64) {
        self.extract_lineage_from_its_location_locally_coherent(reference, event_time, habitat)
    }

    #[must_use]
    #[debug_ensures(
        self.get_active_local_lineage_references_at_location_unordered(
            &old(indexed_location.location().clone()), old(habitat)
        ).last() == Some(&ret),
        "lineage is now indexed unordered at indexed_location.location()"
    )]
    #[debug_ensures(
        old(self.get_active_local_lineage_references_at_location_unordered(
            indexed_location.location(), old(habitat)
        ).len() + 1) == self.get_active_local_lineage_references_at_location_unordered(
            &old(indexed_location.location().clone()), old(habitat)
        ).len(),
        "unordered active lineage index at given location has grown by 1"
    )]
    fn immigrate_globally_coherent(
        &mut self,
        habitat: &H,
        global_reference: GlobalLineageReference,
        indexed_location: IndexedLocation,
        time_of_emigration: f64,
    ) -> R {
        self.immigrate_locally_coherent(
            habitat,
            global_reference,
            indexed_location,
            time_of_emigration,
        )
    }
}
