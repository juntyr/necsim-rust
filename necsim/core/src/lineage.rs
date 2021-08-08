use core::{
    cmp::{Ord, Ordering},
    fmt,
};

use serde::{Deserialize, Deserializer, Serialize, Serializer};

use necsim_core_bond::{NonNegativeF64, NonZeroOneU64, PositiveF64};

use crate::{
    cogs::{BackedUp, Backup, CoalescenceRngSample, Habitat, LineageReference},
    landscape::{IndexedLocation, Location},
};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct GlobalLineageReference(NonZeroOneU64);

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
unsafe impl rsmpi::traits::Equivalence for GlobalLineageReference {
    type Out = rsmpi::datatype::SystemDatatype;

    fn equivalent_datatype() -> Self::Out {
        use rsmpi::raw::FromRaw;

        unsafe { rsmpi::datatype::DatatypeRef::from_raw(rsmpi::ffi::RSMPI_UINT64_T) }
    }
}

#[contract_trait]
impl Backup for GlobalLineageReference {
    unsafe fn backup_unchecked(&self) -> Self {
        GlobalLineageReference(self.0)
    }
}

impl<H: Habitat> LineageReference<H> for GlobalLineageReference {}

#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Clone, Serialize, Deserialize, PartialOrd, Ord)]
#[repr(transparent)]
pub struct LineageInteraction(u64);

impl LineageInteraction {
    #[allow(non_upper_case_globals)]
    pub const Maybe: Self = Self(1_u64);
    #[allow(non_upper_case_globals)]
    pub const None: Self = Self(0_u64);

    #[allow(non_snake_case, clippy::needless_pass_by_value)]
    #[must_use]
    pub const fn Coalescence(parent: GlobalLineageReference) -> Self {
        Self(parent.0.get())
    }

    #[must_use]
    pub const fn is_coalescence(&self) -> bool {
        self.0 > Self::Maybe.0
    }

    #[must_use]
    pub const fn parent(&self) -> Option<GlobalLineageReference> {
        match NonZeroOneU64::new(self.0) {
            Ok(parent) => Some(GlobalLineageReference(parent)),
            Err(_) => None,
        }
    }
}

impl From<Option<GlobalLineageReference>> for LineageInteraction {
    fn from(optional_coalescence: Option<GlobalLineageReference>) -> Self {
        match optional_coalescence {
            None => Self::None,
            Some(coalescence) => Self::Coalescence(coalescence),
        }
    }
}

// Note: manually implementing PartialEq and Eq disables pattern matching
impl PartialEq for LineageInteraction {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl Eq for LineageInteraction {}

impl core::hash::Hash for LineageInteraction {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[repr(C)]
pub struct Lineage {
    pub global_reference: GlobalLineageReference,
    pub last_event_time: NonNegativeF64,
    pub indexed_location: IndexedLocation,
}

#[allow(dead_code)]
const EXCESSIVE_OPTION_LINEAGE_ERROR: [(); 1 - {
    const ASSERT: bool = core::mem::size_of::<Option<Lineage>>() == core::mem::size_of::<Lineage>();
    ASSERT
} as usize] = [];

impl Lineage {
    #[must_use]
    #[debug_ensures(
        ret.indexed_location == old(indexed_location.clone()),
        "stores the indexed_location"
    )]
    #[debug_ensures(ret.last_event_time == 0.0_f64, "starts at t_0 = 0.0")]
    pub fn new<H: Habitat>(indexed_location: IndexedLocation, habitat: &H) -> Self {
        Self {
            global_reference: GlobalLineageReference(unsafe {
                NonZeroOneU64::new_unchecked(
                    habitat.map_indexed_location_to_u64_injective(&indexed_location) + 2,
                )
            }),
            last_event_time: NonNegativeF64::zero(),
            indexed_location,
        }
    }
}

#[allow(clippy::module_name_repetitions)]
#[derive(Debug, PartialEq)]
#[cfg_attr(feature = "mpi", derive(rsmpi::traits::Equivalence))]
pub struct MigratingLineage {
    pub global_reference: GlobalLineageReference,
    pub dispersal_origin: IndexedLocation,
    pub dispersal_target: Location,
    pub prior_time: NonNegativeF64,
    pub event_time: PositiveF64,
    pub coalescence_rng_sample: CoalescenceRngSample,
}

#[contract_trait]
impl Backup for MigratingLineage {
    unsafe fn backup_unchecked(&self) -> Self {
        Self {
            global_reference: self.global_reference.backup_unchecked(),
            dispersal_origin: self.dispersal_origin.clone(),
            dispersal_target: self.dispersal_target.clone(),
            prior_time: self.prior_time,
            event_time: self.event_time,
            coalescence_rng_sample: self.coalescence_rng_sample.backup_unchecked(),
        }
    }
}

impl Ord for MigratingLineage {
    fn cmp(&self, other: &Self) -> Ordering {
        // Order `MigratingLineage`s in lexicographical order:
        //  (1) event_time                       /=\
        //  (2) dispersal_origin        different | events
        //  (3) dispersal_target                 \=/
        //  (4) prior_time              parent + offspring
        //  (5) global_lineage_reference
        //  (6) coalescence_rng_sample
        match self.event_time.cmp(&other.event_time) {
            Ordering::Equal => match (&self.dispersal_origin, &self.dispersal_target)
                .cmp(&(&other.dispersal_origin, &other.dispersal_target))
            {
                Ordering::Equal => match self.prior_time.cmp(&other.prior_time) {
                    Ordering::Equal => (&self.global_reference, &self.coalescence_rng_sample)
                        .cmp(&(&other.global_reference, &other.coalescence_rng_sample)),
                    ordering => ordering,
                },
                ordering => ordering,
            },
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
        self.eq(&**other)
    }
}
