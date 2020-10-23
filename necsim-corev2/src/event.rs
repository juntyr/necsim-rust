use crate::cogs::{Habitat, LineageReference};
use crate::landscape::Location;

pub struct Event<H: Habitat, R: LineageReference<H>> {
    time: f64,
    lineage_reference: R,
    r#type: EventType<H, R>,
    _marker: std::marker::PhantomData<H>,
}

impl<H: Habitat, R: LineageReference<H>> Event<H, R> {
    #[must_use]
    #[allow(clippy::float_cmp)]
    //#[debug_ensures(ret.r#type() == &r#type, "stores r#type")]
    //#[debug_ensures(ret.lineage_reference() == &lineage_reference, "stores lineage_reference")]
    #[debug_ensures(ret.time() == time, "stores time")]
    pub fn new(time: f64, lineage_reference: R, r#type: EventType<H, R>) -> Self {
        Self {
            time,
            lineage_reference,
            r#type,
            _marker: std::marker::PhantomData::<H>,
        }
    }

    #[must_use]
    pub fn time(&self) -> f64 {
        self.time
    }

    #[must_use]
    pub fn lineage_reference(&self) -> &R {
        &self.lineage_reference
    }

    #[must_use]
    pub fn r#type(&self) -> &EventType<H, R> {
        &self.r#type
    }
}

#[allow(clippy::module_name_repetitions)]
pub enum EventType<H: Habitat, R: LineageReference<H>> {
    Speciation,
    Dispersal {
        origin: Location,
        target: Location,
        coalescence: Option<R>,
        _marker: std::marker::PhantomData<H>,
    },
}
