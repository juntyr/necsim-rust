use core::{
    cmp::{Ord, Ordering},
    fmt,
};

use necsim_core_bond::NonZeroOneU64;
use rustacuda_core::DeviceCopy;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::{
    cogs::{BackedUp, Backup, CoalescenceRngSample, Habitat, LineageReference},
    landscape::{IndexedLocation, Location},
};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct GlobalLineageReference(NonZeroOneU64);

#[cfg(feature = "cuda")]
unsafe impl DeviceCopy for GlobalLineageReference {}

impl fmt::Display for GlobalLineageReference {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0.get() - 2)
    }
}

impl<'de> Deserialize<'de> for GlobalLineageReference {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let inner = u64::deserialize(deserializer)?;

        Ok(Self(unsafe { NonZeroOneU64::new_unchecked(inner + 2) }))
    }
}

impl Serialize for GlobalLineageReference {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        (self.0.get() - 2).serialize(serializer)
    }
}

#[cfg(feature = "mpi")]
unsafe impl mpi::traits::Equivalence for GlobalLineageReference {
    type Out = mpi::datatype::SystemDatatype;

    fn equivalent_datatype() -> Self::Out {
        use mpi::raw::FromRaw;

        unsafe { mpi::datatype::DatatypeRef::from_raw(mpi::ffi::RSMPI_UINT64_T) }
    }
}

#[contract_trait]
impl Backup for GlobalLineageReference {
    unsafe fn backup_unchecked(&self) -> Self {
        GlobalLineageReference(self.0)
    }
}

impl<H: Habitat> LineageReference<H> for GlobalLineageReference {}

#[cfg_attr(feature = "cuda", derive(DeviceCopy))]
#[derive(Debug, Clone)]
pub struct Lineage {
    global_reference: GlobalLineageReference,
    indexed_location: Option<IndexedLocation>,
    last_event_time: f64,
}

impl Lineage {
    #[must_use]
    #[debug_ensures(
        ret.indexed_location() == Some(&old(indexed_location.clone())),
        "stores the indexed_location"
    )]
    #[debug_ensures(ret.last_event_time() == 0.0_f64, "starts at t_0 = 0.0")]
    pub fn new<H: Habitat>(indexed_location: IndexedLocation, habitat: &H) -> Self {
        Self {
            global_reference: GlobalLineageReference(unsafe {
                NonZeroOneU64::new_unchecked(
                    habitat.map_indexed_location_to_u64_injective(&indexed_location) + 2,
                )
            }),
            indexed_location: Some(indexed_location),
            last_event_time: 0.0_f64,
        }
    }

    #[must_use]
    pub fn immigrate(
        global_reference: GlobalLineageReference,
        indexed_location: IndexedLocation,
        time_of_emigration: f64,
    ) -> Self {
        Self {
            global_reference,
            indexed_location: Some(indexed_location),
            last_event_time: time_of_emigration,
        }
    }

    #[must_use]
    pub fn emigrate(self) -> GlobalLineageReference {
        self.global_reference
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
    pub fn last_event_time(&self) -> f64 {
        self.last_event_time
    }

    #[must_use]
    pub fn global_reference(&self) -> &GlobalLineageReference {
        &self.global_reference
    }

    /// # Safety
    /// This method should only be called by internal `LineageStore` code to
    /// update the state of the lineages being simulated.
    #[debug_requires(self.is_active(), "lineage must be active to be deactivated")]
    #[debug_requires(event_time > self.last_event_time(), "event_time is after the last event")]
    #[debug_ensures(!self.is_active(), "lineages has been deactivated")]
    #[debug_ensures(self.last_event_time().to_bits() == old(event_time.to_bits()), "updates the last_event_time")]
    #[debug_ensures(
        ret == old(self.indexed_location.as_ref().unwrap().clone()),
        "returns the individual's prior indexed_location"
    )]
    pub unsafe fn remove_from_location(&mut self, event_time: f64) -> IndexedLocation {
        self.last_event_time = event_time;

        self.indexed_location.take().unwrap_unchecked()
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
}

#[allow(clippy::module_name_repetitions)]
#[derive(Debug, PartialEq)]
#[cfg_attr(feature = "mpi", derive(mpi::traits::Equivalence))]
pub struct MigratingLineage {
    pub global_reference: GlobalLineageReference,
    pub dispersal_origin: IndexedLocation,
    pub dispersal_target: Location,
    pub event_time: f64,
    pub coalescence_rng_sample: CoalescenceRngSample,
}

#[contract_trait]
impl Backup for MigratingLineage {
    unsafe fn backup_unchecked(&self) -> Self {
        Self {
            global_reference: self.global_reference.backup_unchecked(),
            dispersal_origin: self.dispersal_origin.clone(),
            dispersal_target: self.dispersal_target.clone(),
            event_time: self.event_time,
            coalescence_rng_sample: self.coalescence_rng_sample.backup_unchecked(),
        }
    }
}

impl Ord for MigratingLineage {
    fn cmp(&self, other: &Self) -> Ordering {
        // Order `MigratingLineage`s in lexicographical order:
        //  (1) event_time
        //  (2) dispersal_target
        //  (3) dispersal_origin
        //  (4) global_reference
        //  (5) coalescence_rng_sample
        match self.event_time.total_cmp(&other.event_time) {
            Ordering::Equal => (
                &self.dispersal_target,
                &self.dispersal_origin,
                &self.global_reference,
                &self.coalescence_rng_sample,
            )
                .cmp(&(
                    &other.dispersal_target,
                    &other.dispersal_origin,
                    &other.global_reference,
                    &other.coalescence_rng_sample,
                )),
            ordering => ordering,
        }
    }
}

impl PartialOrd for MigratingLineage {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Eq for MigratingLineage {}

impl PartialEq<BackedUp<Self>> for MigratingLineage {
    fn eq(&self, other: &BackedUp<Self>) -> bool {
        use core::ops::Deref;
        self.eq(other.deref())
    }
}
