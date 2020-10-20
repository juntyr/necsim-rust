use std::ops::Index;

pub mod global_store;

pub mod active_list;
pub mod gillespie;

use necsim_core::landscape::Location;
use necsim_core::lineage::{Lineage, LineageReference};
use necsim_core::rng::Rng;

#[allow(clippy::inline_always, clippy::inline_fn_without_body)]
#[contract_trait]
pub trait LineageSampler<L: LineageReference>: Sized + Index<L, Output = Lineage> {
    #[must_use]
    fn number_active_lineages(&self) -> usize;

    #[must_use]
    #[debug_ensures(match ret {
        Some(_) => self.number_active_lineages() == old(self.number_active_lineages()) - 1,
        None => old(self.number_active_lineages()) == 0,
    }, "removes an active lineage if some left")]
    #[debug_ensures(
        ret.is_some() -> ret.as_ref().unwrap().1 > time,
        "event occurs later than time"
    )]
    fn pop_next_active_lineage_reference_and_event_time(
        &mut self,
        time: f64,
        rng: &mut impl Rng,
    ) -> Option<(L, f64)>;

    #[debug_ensures(
        self.number_active_lineages() == old(self.number_active_lineages()) + 1,
        "an active lineage was added"
    )]
    #[debug_ensures(
        self[old(reference.clone())].location() == &old(location.clone()),
        "lineage was added to location"
    )]
    fn add_lineage_reference_to_location(
        &mut self,
        reference: L,
        location: Location,
        time: f64,
        rng: &mut impl Rng,
    );
}
