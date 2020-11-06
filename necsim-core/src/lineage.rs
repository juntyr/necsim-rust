use crate::landscape::Location;

#[cfg_attr(feature = "cuda", derive(DeviceCopy))]
#[derive(Debug)]
struct LineageLocation {
    location: Location,
    index_at_location: usize,
}

#[cfg_attr(feature = "cuda", derive(DeviceCopy))]
#[derive(Debug)]
pub struct Lineage {
    location: Option<LineageLocation>,
    time_of_last_event: f64,
}

impl Lineage {
    #[must_use]
    #[debug_ensures(ret.location() == Some(&old(location.clone())), "stores the location")]
    #[debug_ensures(
        ret.index_at_location() == Some(index_at_location),
        "stores the index_at_location"
    )]
    #[debug_ensures(ret.time_of_last_event() == 0.0_f64, "starts at t_0 = 0.0")]
    pub fn new(location: Location, index_at_location: usize) -> Self {
        Self {
            location: Some(LineageLocation {
                location,
                index_at_location,
            }),
            time_of_last_event: 0.0_f64,
        }
    }

    #[must_use]
    pub fn is_active(&self) -> bool {
        self.location.is_some()
    }

    #[must_use]
    pub fn location(&self) -> Option<&Location> {
        self.location.as_ref().map(|l| &l.location)
    }

    #[must_use]
    pub fn index_at_location(&self) -> Option<usize> {
        self.location.as_ref().map(|l| l.index_at_location)
    }

    #[must_use]
    pub fn time_of_last_event(&self) -> f64 {
        self.time_of_last_event
    }

    /// # Safety
    /// This method should only be called by internal `LineageStore` code to update the
    /// state of the lineages being simulated.
    #[debug_requires(self.is_active(), "lineage must be active to be deactivated")]
    #[debug_ensures(!self.is_active(), "lineages has been deactivated")]
    #[debug_ensures(
        ret == old(self.location().unwrap().clone()),
        "returns the individual's prior location"
    )]
    pub unsafe fn remove_from_location(&mut self) -> Location {
        match self.location.take() {
            Some(location) => location.location,
            None => unreachable!(),
        }
    }

    /// # Safety
    /// This method should only be called by internal `LineageStore` code to update the
    /// state of the lineages being simulated.
    #[debug_requires(!self.is_active(), "lineage must be inactive to move")]
    #[debug_ensures(self.location() == Some(&old(location.clone())), "updates the location")]
    #[debug_ensures(
        self.index_at_location() == Some(index_at_location),
        "updates the index_at_location"
    )]
    pub unsafe fn move_to_location(&mut self, location: Location, index_at_location: usize) {
        self.location = Some(LineageLocation {
            location,
            index_at_location,
        });
    }

    /// # Safety
    /// This method should only be called by internal `LineageStore` code to update the
    /// state of the lineages being simulated.
    #[debug_ensures(
        self.index_at_location() == Some(index_at_location),
        "updates the index_at_location"
    )]
    pub unsafe fn update_index_at_location(&mut self, index_at_location: usize) {
        if let Some(ref mut location) = self.location {
            location.index_at_location = index_at_location;
        }
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
