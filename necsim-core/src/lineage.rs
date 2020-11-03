use crate::landscape::Location;

#[cfg_attr(feature = "cuda", derive(DeviceCopy))]
pub struct Lineage {
    location: Location,
    index_at_location: usize,
    time_of_last_event: f64,
}

impl Lineage {
    #[must_use]
    #[debug_ensures(ret.location() == &old(location.clone()), "stores the location")]
    #[debug_ensures(ret.index_at_location() == index_at_location, "stores the index_at_location")]
    #[debug_ensures(ret.time_of_last_event() == 0.0_f64, "starts at t_0 = 0.0")]
    pub fn new(location: Location, index_at_location: usize) -> Self {
        Self {
            location,
            index_at_location,
            time_of_last_event: 0.0_f64,
        }
    }

    #[must_use]
    pub fn location(&self) -> &Location {
        &self.location
    }

    #[must_use]
    pub fn index_at_location(&self) -> usize {
        self.index_at_location
    }

    #[must_use]
    pub fn time_of_last_event(&self) -> f64 {
        self.time_of_last_event
    }

    /// # Safety
    /// This method should only be called by internal `LineageStore` code to update the
    /// state of the lineages being simulated.
    #[debug_ensures(self.location() == &old(location.clone()), "updates the location")]
    #[debug_ensures(self.index_at_location() == index_at_location, "updates the index_at_location")]
    pub unsafe fn move_to_location(&mut self, location: Location, index_at_location: usize) {
        self.location = location;

        self.update_index_at_location(index_at_location);
    }

    /// # Safety
    /// This method should only be called by internal `LineageStore` code to update the
    /// state of the lineages being simulated.
    #[debug_ensures(self.index_at_location() == index_at_location, "updates the index_at_location")]
    pub unsafe fn update_index_at_location(&mut self, index_at_location: usize) {
        self.index_at_location = index_at_location;
    }

    /// # Safety
    /// This method should only be called by internal `LineageStore` code to update the
    /// state of the lineages being simulated.
    #[allow(clippy::float_cmp)]
    #[debug_requires(event_time > self.time_of_last_event(), "event_time is after the last event")]
    #[debug_ensures(self.time_of_last_event() == old(event_time), "updates the time_of_last_event")]
    pub unsafe fn update_time_of_last_event(&mut self, event_time: f64) {
        self.time_of_last_event = event_time;
    }
}
