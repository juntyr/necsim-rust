use crate::landscape::IndexedLocation;

#[cfg_attr(feature = "cuda", derive(DeviceCopy))]
#[derive(Debug)]
pub struct Lineage {
    indexed_location: Option<IndexedLocation>,
    time_of_last_event: f64,
}

impl Lineage {
    #[must_use]
    #[debug_ensures(
        ret.indexed_location() == Some(&old(indexed_location.clone())),
        "stores the indexed_location"
    )]
    #[debug_ensures(ret.time_of_last_event() == 0.0_f64, "starts at t_0 = 0.0")]
    pub fn new(indexed_location: IndexedLocation) -> Self {
        Self {
            indexed_location: Some(indexed_location),
            time_of_last_event: 0.0_f64,
        }
    }

    #[must_use]
    pub fn is_active(&self) -> bool {
        self.indexed_location.is_some()
    }

    #[must_use]
    pub fn indexed_location(&self) -> Option<&IndexedLocation> {
        self.indexed_location.as_ref()
    }

    #[must_use]
    pub fn time_of_last_event(&self) -> f64 {
        self.time_of_last_event
    }

    /// # Safety
    /// This method should only be called by internal `LineageStore` code to
    /// update the state of the lineages being simulated.
    #[debug_requires(self.is_active(), "lineage must be active to be deactivated")]
    #[debug_ensures(!self.is_active(), "lineages has been deactivated")]
    #[debug_ensures(
        ret == old(self.indexed_location.as_ref().unwrap().clone()),
        "returns the individual's prior indexed_location"
    )]
    pub unsafe fn remove_from_location(&mut self) -> IndexedLocation {
        match self.indexed_location.take() {
            Some(indexed_location) => indexed_location,
            None => unreachable!(),
        }
    }

    /// # Safety
    /// This method should only be called by internal `LineageStore` code to
    /// update the state of the lineages being simulated.
    #[debug_requires(!self.is_active(), "lineage must be inactive to move")]
    #[debug_ensures(
        self.indexed_location() == Some(&old(indexed_location.clone())),
        "updates the indexed_location"
    )]
    pub unsafe fn move_to_indexed_location(&mut self, indexed_location: IndexedLocation) {
        self.indexed_location = Some(indexed_location);
    }

    /// # Safety
    /// This method should only be called by internal `LineageStore` code to
    /// update the state of the lineages being simulated.
    #[debug_requires(self.is_active())]
    #[debug_ensures(
        self.indexed_location.as_ref().unwrap().index() == index_at_location,
        "updates the index_at_location"
    )]
    pub unsafe fn update_index_at_location(&mut self, index_at_location: u32) {
        if let Some(ref mut indexed_location) = self.indexed_location {
            indexed_location.index = core::num::NonZeroU32::new_unchecked(index_at_location + 1);
        }
    }

    /// # Safety
    /// This method should only be called by internal `LineageStore` code to
    /// update the state of the lineages being simulated.
    #[allow(clippy::float_cmp)]
    #[debug_requires(event_time > self.time_of_last_event(), "event_time is after the last event")]
    #[debug_ensures(self.time_of_last_event() == old(event_time), "updates the time_of_last_event")]
    pub unsafe fn update_time_of_last_event(&mut self, event_time: f64) {
        self.time_of_last_event = event_time;
    }
}
