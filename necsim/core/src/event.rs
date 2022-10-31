use necsim_core_bond::{NonNegativeF64, PositiveF64};
use serde::{Deserialize, Serialize};

use core::{
    cmp::{Ord, Ordering},
    hash::{Hash, Hasher},
};

use crate::{
    landscape::IndexedLocation,
    lineage::{GlobalLineageReference, LineageInteraction},
};

#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackedEvent {
    global_lineage_reference: GlobalLineageReference,
    prior_time: NonNegativeF64, // time of the previous event
    event_time: PositiveF64,    // time of this event
    origin: IndexedLocation,
    target: Option<IndexedLocation>,
    interaction: LineageInteraction,
}

impl PackedEvent {
    #[must_use]
    #[inline]
    pub fn event_time(&self) -> PositiveF64 {
        self.event_time
    }
}

#[allow(dead_code)]
const EXCESSIVE_OPTION_PACKED_EVENT_ERROR: [(); 1 - {
    const ASSERT: bool =
        core::mem::size_of::<Option<PackedEvent>>() == core::mem::size_of::<PackedEvent>();
    ASSERT
} as usize] = [];

#[allow(dead_code)]
const EXCESSIVE_PACKED_EVENT_ERROR: [(); 1 - {
    const ASSERT: bool = {
        const SPECIATION_SIZE: usize = core::mem::size_of::<SpeciationEvent>();
        const DISPERSAL_SIZE: usize = core::mem::size_of::<DispersalEvent>();

        if SPECIATION_SIZE > DISPERSAL_SIZE {
            SPECIATION_SIZE
        } else {
            DISPERSAL_SIZE
        }
    } == core::mem::size_of::<PackedEvent>();
    ASSERT
} as usize] = [];

#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum EventType {
    Speciation,
    Dispersal(Dispersal),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Dispersal {
    pub target: IndexedLocation,
    pub interaction: LineageInteraction,
}

#[allow(clippy::module_name_repetitions)]
#[allow(clippy::unsafe_derive_deserialize)]
#[derive(Debug, Clone, Serialize, Deserialize, TypeLayout)]
#[repr(C)]
pub struct SpeciationEvent {
    pub global_lineage_reference: GlobalLineageReference,
    pub prior_time: NonNegativeF64, // time of the previous event
    pub event_time: PositiveF64,    // time of this event
    pub origin: IndexedLocation,
}

#[allow(dead_code)]
const EXCESSIVE_OPTION_SPECIATION_EVENT_ERROR: [(); 1 - {
    const ASSERT: bool =
        core::mem::size_of::<Option<SpeciationEvent>>() == core::mem::size_of::<SpeciationEvent>();
    ASSERT
} as usize] = [];

#[allow(clippy::module_name_repetitions)]
#[allow(clippy::unsafe_derive_deserialize)]
#[derive(Debug, Clone, Serialize, Deserialize, TypeLayout)]
#[repr(C)]
pub struct DispersalEvent {
    pub global_lineage_reference: GlobalLineageReference,
    pub prior_time: NonNegativeF64, // time of the previous event
    pub event_time: PositiveF64,    // time of this event
    pub origin: IndexedLocation,
    pub target: IndexedLocation,
    pub interaction: LineageInteraction,
}

#[allow(dead_code)]
const EXCESSIVE_OPTION_DISPERSAL_EVENT_ERROR: [(); 1 - {
    const ASSERT: bool =
        core::mem::size_of::<Option<DispersalEvent>>() == core::mem::size_of::<DispersalEvent>();
    ASSERT
} as usize] = [];

#[allow(clippy::module_name_repetitions)]
pub enum TypedEvent {
    Speciation(SpeciationEvent),
    Dispersal(DispersalEvent),
}

impl From<SpeciationEvent> for PackedEvent {
    fn from(event: SpeciationEvent) -> Self {
        Self {
            global_lineage_reference: event.global_lineage_reference,
            prior_time: event.prior_time,
            event_time: event.event_time,
            origin: event.origin,
            target: None,
            interaction: LineageInteraction::None,
        }
    }
}

impl From<DispersalEvent> for PackedEvent {
    fn from(event: DispersalEvent) -> Self {
        Self {
            global_lineage_reference: event.global_lineage_reference,
            prior_time: event.prior_time,
            event_time: event.event_time,
            origin: event.origin,
            target: Some(event.target),
            interaction: event.interaction,
        }
    }
}

impl From<TypedEvent> for PackedEvent {
    fn from(event: TypedEvent) -> Self {
        match event {
            TypedEvent::Speciation(event) => event.into(),
            TypedEvent::Dispersal(event) => event.into(),
        }
    }
}

impl From<PackedEvent> for TypedEvent {
    fn from(event: PackedEvent) -> Self {
        #[allow(clippy::option_if_let_else)]
        if let Some(target) = event.target {
            Self::Dispersal(DispersalEvent {
                global_lineage_reference: event.global_lineage_reference,
                prior_time: event.prior_time,
                event_time: event.event_time,
                origin: event.origin,
                target,
                interaction: event.interaction,
            })
        } else {
            Self::Speciation(SpeciationEvent {
                global_lineage_reference: event.global_lineage_reference,
                prior_time: event.prior_time,
                event_time: event.event_time,
                origin: event.origin,
            })
        }
    }
}

impl Eq for PackedEvent {}

impl PartialEq for PackedEvent {
    // `Event`s are equal when they have the same `origin`, `event_time`,
    //  `target` and `interaction`
    // (`global_lineage_reference` and `prior_time` are ignored)
    fn eq(&self, other: &Self) -> bool {
        self.origin == other.origin
            && self.event_time == other.event_time
            && self.target == other.target
            && self.interaction == other.interaction
    }
}

impl Ord for PackedEvent {
    fn cmp(&self, other: &Self) -> Ordering {
        // Order `Event`s in lexicographical order:
        //  (1) event_time                       /=\
        //  (2) origin                  different | events
        //  (3) target and interaction           \=/
        //  (4) prior_time              parent + offspring
        //  (5) global_lineage_reference

        (
            &self.event_time,
            &self.origin,
            &self.target,
            &self.interaction,
            &self.prior_time,
            &self.global_lineage_reference,
        )
            .cmp(&(
                &other.event_time,
                &other.origin,
                &other.target,
                &other.interaction,
                &other.prior_time,
                &other.global_lineage_reference,
            ))
    }
}

impl PartialOrd for PackedEvent {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Hash for PackedEvent {
    // `Event`s are equal when they have the same `origin`, `event_time`,
    //  `target` and `interaction`
    // (`global_lineage_reference` and `prior_time` are ignored)
    fn hash<S: Hasher>(&self, state: &mut S) {
        self.origin.hash(state);
        self.event_time.hash(state);
        self.target.hash(state);
        self.interaction.hash(state);
    }
}

impl Eq for SpeciationEvent {}

impl PartialEq for SpeciationEvent {
    // `SpeciationEvent`s are equal when they have the same `origin` and
    //  `event_time` (`global_lineage_reference` and `prior_time` are ignored)
    fn eq(&self, other: &Self) -> bool {
        self.origin == other.origin && self.event_time == other.event_time
    }
}

impl Eq for DispersalEvent {}

impl PartialEq for DispersalEvent {
    // `SpeciationEvent`s are equal when they have the same `origin`,
    //  `event_time`, `target` and `interaction`
    //  (`global_lineage_reference` and `prior_time` are ignored)
    fn eq(&self, other: &Self) -> bool {
        self.origin == other.origin
            && self.event_time == other.event_time
            && self.target == other.target
            && self.interaction == other.interaction
    }
}
