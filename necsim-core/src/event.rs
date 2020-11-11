use core::hash::{Hash, Hasher};
use core::marker::PhantomData;

use crate::cogs::{Habitat, LineageReference};
use crate::landscape::IndexedLocation;

#[cfg(feature = "cuda")]
use rustacuda_core::DeviceCopy;

#[cfg(feature = "cuda")]
use rust_cuda::common::RustToCuda;

pub struct Event<H: Habitat, R: LineageReference<H>> {
    time: f64,
    lineage_reference: R,
    r#type: EventType<H, R>,
    marker: PhantomData<H>,
}

impl<H: Habitat, R: LineageReference<H>> Eq for Event<H, R> {}

impl<H: Habitat, R: LineageReference<H>> PartialEq for Event<H, R> {
    fn eq(&self, other: &Self) -> bool {
        self.time == other.time && self.r#type == other.r#type
    }
}

impl<H: Habitat, R: LineageReference<H>> Hash for Event<H, R> {
    fn hash<S: Hasher>(&self, state: &mut S) {
        self.time.to_bits().hash(state);
        self.r#type.hash(state);
    }
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
            marker: PhantomData::<H>,
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
        marker: PhantomData<H>,
    },
}

impl<H: Habitat, R: LineageReference<H>> Eq for EventType<H, R> {}

impl<H: Habitat, R: LineageReference<H>> PartialEq for EventType<H, R> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (EventType::Speciation, EventType::Speciation) => true,
            (
                EventType::Dispersal {
                    origin: self_origin,
                    target: self_target,
                    ..
                },
                EventType::Dispersal {
                    origin: other_origin,
                    target: other_target,
                    ..
                },
            ) => self_origin == other_origin && self_target == other_target,
            _ => false,
        }
    }
}

impl<H: Habitat, R: LineageReference<H>> Hash for EventType<H, R> {
    fn hash<S: Hasher>(&self, state: &mut S) {
        core::mem::discriminant(self).hash(state);

        if let EventType::Dispersal { origin, target, .. } = self {
            origin.hash(state);
            target.hash(state);
        }
    }
}

impl<H: Habitat, R: LineageReference<H>> Clone for Event<H, R> {
    fn clone(&self) -> Self {
        Self {
            time: self.time,
            lineage_reference: self.lineage_reference.clone(),
            r#type: self.r#type.clone(),
            marker: self.marker,
        }
    }
}

impl<H: Habitat, R: LineageReference<H>> Clone for EventType<H, R> {
    fn clone(&self) -> Self {
        match self {
            EventType::Speciation => EventType::Speciation,
            EventType::Dispersal {
                origin,
                target,
                coalescence,
                marker,
            } => EventType::Dispersal {
                origin: origin.clone(),
                target: target.clone(),
                coalescence: coalescence.clone(),
                marker: *marker,
            },
        }
    }
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
            .field("marker", &format_args!("PhantomData"))
            .finish()
    }
}

#[cfg(feature = "cuda")]
unsafe impl<H: Habitat + RustToCuda, R: LineageReference<H> + DeviceCopy> DeviceCopy
    for Event<H, R>
{
}

#[cfg(feature = "cuda")]
unsafe impl<H: Habitat + RustToCuda, R: LineageReference<H> + DeviceCopy> DeviceCopy
    for EventType<H, R>
{
}
