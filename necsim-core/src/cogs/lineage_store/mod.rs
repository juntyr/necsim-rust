use core::ops::Index;

use super::{Habitat, LineageReference};
use crate::landscape::Location;
use crate::lineage::Lineage;

mod contract;

#[allow(clippy::inline_always, clippy::inline_fn_without_body)]
#[contract_trait]
pub trait LineageStore<H: Habitat, R: LineageReference<H>>:
    Sized + Index<R, Output = Lineage>
{
    #[must_use]
    #[allow(clippy::float_cmp)]
    #[debug_ensures(if sample_percentage == 0.0_f64 {
        ret.get_number_total_lineages() == 0
    } else if sample_percentage == 1.0_f64 {
        ret.get_number_total_lineages() == habitat.get_total_habitat()
    } else {
        true
    }, "samples active lineages according to settings.sample_percentage()")]
    fn new(sample_percentage: f64, habitat: &H) -> Self;

    #[must_use]
    fn get_number_total_lineages(&self) -> usize;

    #[must_use]
    fn get_active_lineages_at_location(&self, location: &Location) -> &[R];

    #[must_use]
    #[allow(clippy::double_parens)]
    #[debug_ensures(ret.is_some() -> (
        (&self[old(reference.clone())] as *const Lineage) == (ret.unwrap() as *const Lineage)
    ), "provides the checked version of the Index<R, Output = Lineage> trait")]
    fn get(&self, reference: R) -> Option<&Lineage>;

    #[debug_requires(self.get(reference.clone()).is_some(), "lineage reference is valid")]
    #[debug_requires(
        !contract::explicit_lineage_store_lineage_at_location_contract(self, reference.clone()),
        "lineage is not at the location and index it references"
    )]
    #[debug_requires(
        contract::explicit_lineage_store_invariant_contract(self, &location),
        "invariant of lineage-location bijection holds"
    )]
    #[debug_ensures(
        self[old(reference.clone())].location() == &old(location.clone()),
        "lineage was added to location"
    )]
    #[debug_ensures(
        contract::explicit_lineage_store_lineage_at_location_contract(
            self, old(reference.clone())
        ), "lineage is at the location and index it references"
    )]
    #[debug_ensures(
        contract::explicit_lineage_store_invariant_contract(self, &old(location.clone())),
        "maintains invariant of lineage-location bijection"
    )]
    fn add_lineage_to_location(&mut self, reference: R, location: Location);

    #[debug_requires(self.get(reference.clone()).is_some(), "lineage reference is valid")]
    #[debug_requires(
        contract::explicit_lineage_store_lineage_at_location_contract(self, reference.clone()),
        "lineage is at the location and index it references"
    )]
    #[debug_requires(
        contract::explicit_lineage_store_invariant_contract(
            self, self[reference.clone()].location()
        ), "invariant of lineage-location bijection holds"
    )]
    #[debug_ensures(
        !contract::explicit_lineage_store_lineage_at_location_contract(
            self, old(reference.clone())
        ), "lineage was removed from the location and index it references"
    )]
    #[debug_ensures(
        contract::explicit_lineage_store_invariant_contract(
            self, self[old(reference.clone())].location()
        ), "maintains invariant of lineage-location bijection"
    )]
    fn remove_lineage_from_its_location(&mut self, reference: R);

    #[allow(clippy::float_cmp)]
    #[debug_requires(self.get(reference.clone()).is_some(), "lineage reference is valid")]
    #[debug_ensures(
        self[old(reference.clone())].time_of_last_event() == old(event_time),
        "updates the time of the last event of the lineage reference"
    )]
    fn update_lineage_time_of_last_event(&mut self, reference: R, event_time: f64);
}
