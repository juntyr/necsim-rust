use crate::{
    cogs::CoalescenceRngSample,
    landscape::{IndexedLocation, Location},
    lineage::GlobalLineageReference,
};

#[allow(clippy::inline_always, clippy::inline_fn_without_body)]
#[contract_trait]
pub trait ImmigrationEntry: core::fmt::Debug {
    #[must_use]
    #[debug_requires(match &optional_next_event_time {
        Some(event_time) => *event_time >= 0.0_f64,
        None => true,
    }, "option_next_event_time is non-negative")]
    #[debug_ensures(match &ret {
        Some(immigration) => immigration.3 >= 0.0,
        None => true,
    }, "immigration event time is non-negative")]
    #[debug_ensures(match (&ret, old(optional_next_event_time)) {
        (Some(immigration), Some(event_time)) => immigration.3 <= event_time,
        _ => true,
    }, "immigration event time is before the next event")]
    fn next_optional_immigration(
        &mut self,
        optional_next_event_time: Option<f64>,
    ) -> Option<(
        GlobalLineageReference,
        IndexedLocation,
        Location,
        f64,
        CoalescenceRngSample,
    )>;
}
