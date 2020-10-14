use crate::landscape::Location;

pub struct Event {
    time: f64,
    r#type: EventType,
}

impl Event {
    #[must_use]
    pub fn new(time: f64, r#type: EventType) -> Self {
        Self { time, r#type }
    }

    #[must_use]
    pub fn time(&self) -> f64 {
        self.time
    }

    #[must_use]
    pub fn r#type(&self) -> &EventType {
        &self.r#type
    }
}

#[allow(clippy::module_name_repetitions)]
pub enum EventType {
    // TODO: need to store reference to lineage somehow
    Speciation,
    Dispersal {
        origin: Location,
        target: Location,
        // TODO: need to store reference to parent somehow
        coalescence: bool,
    },
}
