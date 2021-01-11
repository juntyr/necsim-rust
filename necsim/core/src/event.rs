use core::hash::{Hash, Hasher};

use crate::{landscape::IndexedLocation, lineage::GlobalLineageReference};

#[derive(Debug, TypeLayout, Clone)]
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
#[derive(Debug, Clone)]
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
