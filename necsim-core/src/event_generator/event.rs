use crate::landscape::Location;
use crate::lineage::LineageReference;

pub struct Event<L: LineageReference> {
    time: f64,
    lineage_reference: L,
    r#type: EventType<L>,
}

impl<L: LineageReference> Event<L> {
    #[must_use]
    #[allow(clippy::float_cmp)]
    //#[debug_ensures(ret.r#type() == &r#type, "stores r#type")]
    //#[debug_ensures(ret.lineage_reference() == &lineage_reference, "stores lineage_reference")]
    #[debug_ensures(ret.time() == time, "stores time")]
    pub fn new(time: f64, lineage_reference: L, r#type: EventType<L>) -> Self {
        Self {
            time,
            lineage_reference,
            r#type,
        }
    }

    #[must_use]
    pub fn time(&self) -> f64 {
        self.time
    }

    #[must_use]
    pub fn lineage_reference(&self) -> &L {
        &self.lineage_reference
    }

    #[must_use]
    pub fn r#type(&self) -> &EventType<L> {
        &self.r#type
    }
}

#[allow(clippy::module_name_repetitions)]
pub enum EventType<L: LineageReference> {
    Speciation,
    Dispersal {
        origin: Location,
        target: Location,
        coalescence: Option<L>,
    },
}
