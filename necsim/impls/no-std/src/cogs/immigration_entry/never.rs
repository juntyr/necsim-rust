use necsim_core::{
    cogs::{CoalescenceRngSample, ImmigrationEntry},
    landscape::{IndexedLocation, Location},
    lineage::GlobalLineageReference,
};

#[allow(clippy::module_name_repetitions)]
#[cfg_attr(feature = "cuda", derive(RustToCuda))]
#[derive(Debug)]
pub struct NeverImmigrationEntry(());

impl Default for NeverImmigrationEntry {
    fn default() -> Self {
        Self(())
    }
}

#[contract_trait]
impl ImmigrationEntry for NeverImmigrationEntry {
    #[must_use]
    #[inline]
    fn next_optional_immigration(
        &mut self,
        _optional_next_event_time: Option<f64>,
    ) -> Option<(
        GlobalLineageReference,
        IndexedLocation,
        Location,
        f64,
        CoalescenceRngSample,
    )> {
        None
    }
}
