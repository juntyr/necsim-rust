use serde::{Deserialize, Serialize};

use core::{
    cmp::{Ord, Ordering},
    hash::{Hash, Hasher},
};

use crate::{landscape::IndexedLocation, lineage::GlobalLineageReference};

#[derive(Debug, TypeLayout, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "cuda", derive(DeviceCopy))]
pub struct Event {
    origin: IndexedLocation,
    time: f64,
    global_lineage_reference: GlobalLineageReference,
    r#type: EventType,
}

impl Eq for Event {}

impl PartialEq for Event {
    // `Event`s are equal when they have the same `origin`, `time` and `r#type`
    // (`global_lineage_reference` is ignored)
    fn eq(&self, other: &Self) -> bool {
        self.origin == other.origin && self.time == other.time && self.r#type == other.r#type
    }
}

impl Ord for Event {
    fn cmp(&self, other: &Self) -> Ordering {
        // Order `Event`s in lexicographical order:
        //  (1) time
        //  (2) origin
        //  (3) r#type (target)
        match self.time.total_cmp(&other.time) {
            Ordering::Equal => (&self.origin, &self.r#type, &self.global_lineage_reference).cmp(&(
                &other.origin,
                &other.r#type,
                &other.global_lineage_reference,
            )),
            ordering => ordering,
        }
    }
}

impl PartialOrd for Event {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Hash for Event {
    // `Event`s are equal when they have the same `origin`, `time` and `r#type`
    // (`global_lineage_reference` is ignored)
    fn hash<S: Hasher>(&self, state: &mut S) {
        self.origin.hash(state);
        self.time.to_bits().hash(state);
        self.r#type.hash(state);
    }
}

impl Event {
    #[must_use]
    #[allow(clippy::float_cmp)]
    //#[debug_ensures(ret.r#type() == &r#type, "stores r#type")]
    //#[debug_ensures(ret.lineage_reference() == &lineage_reference, "stores lineage_reference")]
    #[debug_ensures(ret.time() == time, "stores time")]
    pub fn new(
        origin: IndexedLocation,
        time: f64,
        global_lineage_reference: GlobalLineageReference,
        r#type: EventType,
    ) -> Self {
        Self {
            origin,
            time,
            global_lineage_reference,
            r#type,
        }
    }

    #[must_use]
    pub fn origin(&self) -> &IndexedLocation {
        &self.origin
    }

    #[must_use]
    pub fn time(&self) -> f64 {
        self.time
    }

    #[must_use]
    pub fn global_lineage_reference(&self) -> &GlobalLineageReference {
        &self.global_lineage_reference
    }

    #[must_use]
    pub fn r#type(&self) -> &EventType {
        &self.r#type
    }
}

#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "cuda", derive(DeviceCopy))]
pub enum EventType {
    Speciation,
    Dispersal {
        target: IndexedLocation,
        coalescence: Option<GlobalLineageReference>,
    },
}

impl Eq for EventType {}

impl PartialEq for EventType {
    // `EventType`s are equal when they have the same type and `target`
    // (`coalescence` is ignored)
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (EventType::Speciation, EventType::Speciation) => true,
            (
                EventType::Dispersal {
                    target: self_target,
                    ..
                },
                EventType::Dispersal {
                    target: other_target,
                    ..
                },
            ) => self_target == other_target,
            _ => false,
        }
    }
}

impl Ord for EventType {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (EventType::Speciation, EventType::Speciation) => Ordering::Equal,
            (EventType::Speciation, _) => Ordering::Less,
            (_, EventType::Speciation) => Ordering::Greater,
            (
                EventType::Dispersal {
                    target: self_target,
                    ..
                },
                EventType::Dispersal {
                    target: other_target,
                    ..
                },
            ) => self_target.cmp(other_target),
        }
    }
}

impl PartialOrd for EventType {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Hash for EventType {
    // `EventType`s are equal when they have the same type and `target`
    // (`coalescence` is ignored)
    fn hash<S: Hasher>(&self, state: &mut S) {
        core::mem::discriminant(self).hash(state);

        if let EventType::Dispersal { target, .. } = self {
            target.hash(state);
        }
    }
}
