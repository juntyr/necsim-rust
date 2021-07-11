use core::{convert::TryFrom, fmt, marker::PhantomData, num::NonZeroUsize};

use serde::{
    de::{EnumAccess, Error, VariantAccess, Visitor},
    Deserialize, Deserializer,
};

use super::{AbsoluteCapacity, EventSlice, RelativeCapacity};

#[derive(Deserialize)]
enum EventSliceRaw {
    Absolute(AbsoluteCapacity),
    Relative(RelativeCapacity),
}

// TODO: Remove legacy RON workaround after deprecation
impl<'de> Deserialize<'de> for EventSlice {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        enum EnumField {
            Absolute,
            Relative,
            Legacy(NonZeroUsize),
        }

        struct EnumFieldVisitor;

        impl<'de> Visitor<'de> for EnumFieldVisitor {
            type Value = EnumField;

            fn expecting(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
                fmt.write_str("variant identifier")
            }

            fn visit_u64<E: Error>(self, value: u64) -> Result<Self::Value, E> {
                match usize::try_from(value) {
                    Ok(capacity) => match NonZeroUsize::new(capacity) {
                        Some(capacity) => Ok(EnumField::Legacy(capacity)),
                        None => Err(Error::custom("expected a non-zero value")),
                    },
                    Err(err) => Err(Error::custom(err)),
                }
            }

            fn visit_str<E: Error>(self, value: &str) -> Result<Self::Value, E> {
                match value {
                    "Absolute" => Ok(EnumField::Absolute),
                    "Relative" => Ok(EnumField::Relative),
                    _ => Err(Error::unknown_variant(value, VARIANTS)),
                }
            }

            fn visit_bytes<E: Error>(self, value: &[u8]) -> Result<Self::Value, E> {
                match value {
                    b"Absolute" => Ok(EnumField::Absolute),
                    b"Relative" => Ok(EnumField::Relative),
                    _ => Err(Error::unknown_variant(
                        &alloc::string::String::from_utf8_lossy(value),
                        VARIANTS,
                    )),
                }
            }
        }

        impl<'de> Deserialize<'de> for EnumField {
            #[inline]
            fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
                deserializer.deserialize_identifier(EnumFieldVisitor)
            }
        }

        struct EnumVisitor<'de> {
            marker: PhantomData<EventSlice>,
            lifetime: PhantomData<&'de ()>,
        }

        impl<'de> Visitor<'de> for EnumVisitor<'de> {
            type Value = EventSlice;

            fn expecting(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
                fmt.write_str("enum EventSlice")
            }

            fn visit_enum<A: EnumAccess<'de>>(self, data: A) -> Result<Self::Value, A::Error> {
                match EnumAccess::variant(data)? {
                    (EnumField::Absolute, variant) => Result::map(
                        VariantAccess::newtype_variant::<AbsoluteCapacity>(variant),
                        EventSlice::Absolute,
                    ),
                    (EnumField::Relative, variant) => Result::map(
                        VariantAccess::newtype_variant::<RelativeCapacity>(variant),
                        EventSlice::Relative,
                    ),
                    (EnumField::Legacy(capacity), _variant) => {
                        warn!(
                            "Using the legacy `event_slice` capacity is deprecated. Please use \
                             `Absolute(capacity: {})` instead.",
                            capacity
                        );

                        Ok(EventSlice::Absolute(AbsoluteCapacity { capacity }))
                    },
                }
            }

            fn visit_u64<E: Error>(self, value: u64) -> Result<Self::Value, E> {
                match usize::try_from(value) {
                    Ok(value) => match NonZeroUsize::new(value) {
                        Some(capacity) => Ok(EventSlice::Absolute(AbsoluteCapacity { capacity })),
                        None => Err(Error::custom("expected a non-zero value")),
                    },
                    Err(_err) => Err(Error::custom(alloc::format!(
                        "expected a {} bit value",
                        usize::BITS
                    ))),
                }
            }
        }

        const VARIANTS: &[&str] = &["Absolute", "Relative"];

        deserializer.deserialize_enum(
            "EventSlice",
            VARIANTS,
            EnumVisitor {
                marker: PhantomData::<EventSlice>,
                lifetime: PhantomData,
            },
        )
    }
}
