use core::num::NonZeroU64;

use serde::{Deserialize, Serialize};

use crate::{
    cogs::{Habitat, LineageReference},
    landscape::IndexedLocation,
};

#[cfg_attr(feature = "cuda", derive(DeviceCopy))]
#[derive(Debug, PartialEq, Eq, Hash, Clone, Serialize, Deserialize)]
pub struct GlobalLineageReference(NonZeroU64);

#[cfg(feature = "mpi")]
unsafe impl mpi::traits::Equivalence for GlobalLineageReference {
    type Out = mpi::datatype::SystemDatatype;

    fn equivalent_datatype() -> Self::Out {
        use mpi::raw::FromRaw;

        unsafe { mpi::datatype::DatatypeRef::from_raw(mpi::ffi::RSMPI_UINT64_T) }
    }
}

impl<H: Habitat> LineageReference<H> for GlobalLineageReference {}

#[cfg_attr(feature = "cuda", derive(DeviceCopy))]
#[derive(Debug, Clone)]
pub struct Lineage {
    global_reference: GlobalLineageReference,
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
    pub fn new<H: Habitat>(indexed_location: IndexedLocation, habitat: &H) -> Self {
        Self {
            global_reference: GlobalLineageReference(unsafe {
                NonZeroU64::new_unchecked(
                    habitat.map_indexed_location_to_u64_injective(&indexed_location) + 1,
                )
            }),
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

    #[must_use]
    pub fn global_reference(&self) -> &GlobalLineageReference {
        &self.global_reference
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
    #[debug_ensures(!self.is_active(), "lineages has been deactivated")]
    #[debug_ensures(
        ret.is_some() == old(self.is_active()),
        "returns None iff inactive"
    )]
    #[debug_ensures(
        ret == old(self.indexed_location.clone()),
        "if active, returns the individual's prior indexed_location"
    )]
    pub unsafe fn try_remove_from_location(&mut self) -> Option<IndexedLocation> {
        self.indexed_location.take()
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
    #[allow(clippy::float_cmp)]
    #[debug_requires(event_time > self.time_of_last_event(), "event_time is after the last event")]
    #[debug_ensures(self.time_of_last_event() == old(event_time), "updates the time_of_last_event")]
    pub unsafe fn update_time_of_last_event(&mut self, event_time: f64) {
        self.time_of_last_event = event_time;
    }
}
