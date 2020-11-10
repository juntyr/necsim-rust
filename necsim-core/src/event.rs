use core::marker::PhantomData;

use crate::cogs::{Habitat, LineageReference};
use crate::landscape::IndexedLocation;

pub struct Event<H: Habitat, R: LineageReference<H>> {
    time: f64,
    lineage_reference: R,
    r#type: EventType<H, R>,
    _marker: PhantomData<H>,
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
            _marker: PhantomData::<H>,
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
#[derive(Debug)]
pub enum EventType<H: Habitat, R: LineageReference<H>> {
    Speciation,
    Dispersal {
        origin: IndexedLocation,
        target: IndexedLocation,
        coalescence: Option<R>,
        _marker: PhantomData<H>,
    },
}

impl<H: Habitat, R: LineageReference<H>> core::fmt::Debug for Event<H, R> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Event")
            .field(
                "time",
                &format_args!("{}", ryu::Buffer::new().format(self.time)),
            )
            .field("lineage_reference", &self.lineage_reference)
            .field("type", &self.r#type)
            .field("_marker", &format_args!("PhantomData"))
            .finish()
    }
}
