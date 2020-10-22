use std::ops::Index;

use super::{Habitat, LineageReference};

use crate::landscape::Location;
use crate::lineage::Lineage;
use crate::rng::Rng;

#[allow(clippy::inline_always, clippy::inline_fn_without_body)]
#[contract_trait]
pub trait LineageSampler<H: Habitat, R: LineageReference<H>>:
    Sized + Index<R, Output = Lineage>
{
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
    ) -> Option<(R, f64)>;

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
        reference: R,
        location: Location,
        time: f64,
        rng: &mut impl Rng,
    );
}
