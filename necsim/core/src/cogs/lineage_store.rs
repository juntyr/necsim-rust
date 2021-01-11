use core::ops::Index;

use super::{Habitat, LineageReference};
use crate::{
    landscape::{IndexedLocation, Location},
    lineage::{GlobalLineageReference, Lineage},
};

#[allow(clippy::inline_always, clippy::inline_fn_without_body)]
#[contract_trait]
pub trait LineageStore<H: Habitat, R: LineageReference<H>>:
    Sized + Index<R, Output = Lineage> + core::fmt::Debug
{
    type LineageReferenceIterator<'a>: ExactSizeIterator<Item = R>;

    #[must_use]
    #[debug_ensures(
        ret >= self.iter_local_lineage_references().len(),
        "total number of lineages is at least local number of lineages"
    )]
    fn get_number_total_lineages(&self) -> usize;

    #[must_use]
    fn iter_local_lineage_references(&self) -> Self::LineageReferenceIterator<'_>;

    #[must_use]
    #[allow(clippy::double_parens)]
    #[debug_ensures(ret.is_some() -> (
        core::ptr::eq(&self[old(reference.clone())], ret.unwrap())
    ), "provides the checked version of the Index<R, Output = Lineage> trait")]
    fn get(&self, reference: R) -> Option<&Lineage>;

    #[allow(clippy::float_cmp)]
    #[debug_requires(self.get(reference.clone()).is_some(), "lineage reference is valid")]
    #[debug_ensures(
        self[old(reference.clone())].time_of_last_event() == old(event_time),
        "updates the time of the last event of the lineage reference"
    )]
    fn update_lineage_time_of_last_event(&mut self, reference: R, event_time: f64);
}

#[allow(clippy::inline_always, clippy::inline_fn_without_body)]
#[allow(clippy::module_name_repetitions)]
#[contract_trait]
pub trait CoherentLineageStore<H: Habitat, R: LineageReference<H>>: LineageStore<H, R> {
    type LocationIterator<'a>: Iterator<Item = Location>;

    #[must_use]
    fn iter_active_locations(&self) -> Self::LocationIterator<'_>;

    #[must_use]
    fn get_active_local_lineage_references_at_location_unordered(
        &self,
        location: &Location,
    ) -> &[R];

    #[must_use]
    fn get_active_global_lineage_reference_at_indexed_location(
        &self,
        indexed_location: &IndexedLocation,
    ) -> Option<&GlobalLineageReference>;

    #[debug_requires(self.get(reference.clone()).is_some(), "lineage reference is valid")]
    #[debug_requires(!self[reference.clone()].is_active(), "lineage is inactive")]
    #[debug_ensures(self[old(reference.clone())].is_active(), "lineage was activated")]
    #[debug_ensures(
        self[old(reference.clone())].indexed_location() == Some(&old(indexed_location.clone())),
        "lineage was added to indexed_location"
    )]
    #[debug_ensures(
        self.get_active_global_lineage_reference_at_indexed_location(
            &old(indexed_location.clone())
        ) == Some(self[old(reference.clone())].global_reference()),
        "lineage is now indexed at indexed_location"
    )]
    #[debug_ensures(
        self.get_active_local_lineage_references_at_location_unordered(
            &old(indexed_location.location().clone())
        ).last() == Some(&old(reference.clone())),
        "lineage is now indexed unordered at indexed_location.location()"
    )]
    #[debug_ensures(
        old(self.get_active_local_lineage_references_at_location_unordered(
            indexed_location.location()
        ).len() + 1) == self.get_active_local_lineage_references_at_location_unordered(
            &old(indexed_location.location().clone())
        ).len(),
        "unordered active lineage index at returned location has grown by 1"
    )]
    fn insert_lineage_to_indexed_location_coherent(
        &mut self,
        reference: R,
        indexed_location: IndexedLocation,
    );

    #[must_use]
    #[debug_requires(self.get(reference.clone()).is_some(), "lineage reference is valid")]
    #[debug_requires(self[reference.clone()].is_active(), "lineage is active")]
    #[debug_ensures(!self[old(reference.clone())].is_active(), "lineage was deactivated")]
    #[debug_ensures(
        ret == old(self[reference.clone()].indexed_location().unwrap().clone()),
        "returns the individual's prior IndexedLocation"
    )]
    #[debug_ensures(
        self.get_active_global_lineage_reference_at_indexed_location(&ret).is_none(),
        "lineage is no longer indexed at its prior IndexedLocation"
    )]
    #[debug_ensures(
        self.get_active_local_lineage_references_at_location_unordered(&ret.location()).len() + 1
            == old(self.get_active_local_lineage_references_at_location_unordered(
                self[reference.clone()].indexed_location().unwrap().location()
            ).len()),
        "unordered active lineage index at returned location has shrunk by 1"
    )]
    fn extract_lineage_from_its_location_coherent(&mut self, reference: R) -> IndexedLocation;
}

#[allow(clippy::inline_always, clippy::inline_fn_without_body)]
#[allow(clippy::module_name_repetitions)]
#[contract_trait]
pub trait IncoherentLineageStore<H: Habitat, R: LineageReference<H>>: LineageStore<H, R> {
    #[debug_requires(self.get(reference.clone()).is_some(), "lineage reference is valid")]
    #[debug_requires(!self[reference.clone()].is_active(), "lineage is inactive")]
    #[debug_ensures(self[old(reference.clone())].is_active(), "lineage was activated")]
    #[debug_ensures(
        self[old(reference.clone())].indexed_location() == Some(&old(indexed_location.clone())),
        "lineage was added to indexed_location"
    )]
    fn insert_lineage_to_indexed_location(
        &mut self,
        reference: R,
        indexed_location: IndexedLocation,
    );

    #[must_use]
    #[debug_requires(self.get(reference.clone()).is_some(), "lineage reference is valid")]
    #[debug_requires(self[reference.clone()].is_active(), "lineage is active")]
    #[debug_ensures(!self[old(reference.clone())].is_active(), "lineage was deactivated")]
    #[debug_ensures(
        ret == old(self[reference.clone()].indexed_location().unwrap().clone()),
        "returns the individual's prior location"
    )]
    fn extract_lineage_from_its_location(&mut self, reference: R) -> IndexedLocation;
}
