use core::num::NonZeroU32;

use necsim_core::{
    cogs::{Habitat, MathsCore, RngCore, UniformlySampleableHabitat},
    landscape::{IndexedLocation, LandscapeExtent, Location},
};
use necsim_core_bond::{OffByOneU32, OffByOneU64};

use super::AlmostInfiniteHabitat;

const ALMOST_INFINITE_EXTENT: LandscapeExtent =
    LandscapeExtent::new(Location::new(0, 0), OffByOneU32::max(), OffByOneU32::max());

#[allow(clippy::module_name_repetitions)]
#[derive(Debug)]
#[cfg_attr(feature = "cuda", derive(rust_cuda::lend::LendRustToCuda))]
#[cfg_attr(feature = "cuda", cuda(free = "M"))]
pub struct AlmostInfiniteDownscaledHabitat<M: MathsCore> {
    #[cfg_attr(feature = "cuda", cuda(embed))]
    habitat: AlmostInfiniteHabitat<M>,
    downscale_x: Log2U16,
    downscale_y: Log2U16,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, TypeLayout)]
#[repr(u16)]
pub enum Log2U16 {
    Shl0 = 1 << 0,
    Shl1 = 1 << 1,
    Shl2 = 1 << 2,
    Shl3 = 1 << 3,
    Shl4 = 1 << 4,
    Shl5 = 1 << 5,
    Shl6 = 1 << 6,
    Shl7 = 1 << 7,
    Shl8 = 1 << 8,
    Shl9 = 1 << 9,
    Shl10 = 1 << 10,
    Shl11 = 1 << 11,
    Shl12 = 1 << 12,
    Shl13 = 1 << 13,
    Shl14 = 1 << 14,
    Shl15 = 1 << 15,
}

impl Log2U16 {
    #[must_use]
    pub const fn log2(self) -> u32 {
        match self {
            Self::Shl0 => 0,
            Self::Shl1 => 1,
            Self::Shl2 => 2,
            Self::Shl3 => 3,
            Self::Shl4 => 4,
            Self::Shl5 => 5,
            Self::Shl6 => 6,
            Self::Shl7 => 7,
            Self::Shl8 => 8,
            Self::Shl9 => 9,
            Self::Shl10 => 10,
            Self::Shl11 => 11,
            Self::Shl12 => 12,
            Self::Shl13 => 13,
            Self::Shl14 => 14,
            Self::Shl15 => 15,
        }
    }
}

impl<M: MathsCore> Clone for AlmostInfiniteDownscaledHabitat<M> {
    fn clone(&self) -> Self {
        Self {
            habitat: self.habitat.clone(),
            downscale_x: self.downscale_x,
            downscale_y: self.downscale_y,
        }
    }
}

impl<M: MathsCore> AlmostInfiniteDownscaledHabitat<M> {
    #[must_use]
    pub fn new(downscale_x: Log2U16, downscale_y: Log2U16) -> Self {
        Self::new_with_habitat(AlmostInfiniteHabitat::default(), downscale_x, downscale_y)
    }

    #[must_use]
    pub fn new_with_habitat(
        habitat: AlmostInfiniteHabitat<M>,
        downscale_x: Log2U16,
        downscale_y: Log2U16,
    ) -> Self {
        Self {
            habitat,
            downscale_x,
            downscale_y,
        }
    }

    #[must_use]
    pub fn downscale_x(&self) -> Log2U16 {
        self.downscale_x
    }

    #[must_use]
    pub fn downscale_y(&self) -> Log2U16 {
        self.downscale_y
    }

    #[must_use]
    pub fn downscale_area(&self) -> NonZeroU32 {
        // 2^{0..15} * 2^{0..15} >=1 and < 2^32
        unsafe { NonZeroU32::new_unchecked((self.downscale_x as u32) * (self.downscale_y as u32)) }
    }

    #[must_use]
    pub fn unscaled(&self) -> &AlmostInfiniteHabitat<M> {
        &self.habitat
    }
}

#[contract_trait]
impl<M: MathsCore> Habitat<M> for AlmostInfiniteDownscaledHabitat<M> {
    type LocationIterator<'a> = impl Iterator<Item = Location>;

    #[must_use]
    fn is_finite(&self) -> bool {
        false
    }

    #[must_use]
    fn get_extent(&self) -> &LandscapeExtent {
        &ALMOST_INFINITE_EXTENT
    }

    #[must_use]
    fn get_total_habitat(&self) -> OffByOneU64 {
        OffByOneU64::max()
    }

    #[must_use]
    fn get_habitat_at_location(&self, location: &Location) -> u32 {
        // TODO: optimise
        if ((location.x() % (self.downscale_x as u32)) == 0)
            && ((location.y() % (self.downscale_y as u32)) == 0)
        {
            (self.downscale_x as u32) * (self.downscale_y as u32)
        } else {
            0
        }
    }

    #[must_use]
    fn map_indexed_location_to_u64_injective(&self, indexed_location: &IndexedLocation) -> u64 {
        // TODO: optimise
        let index_x = indexed_location.index() % (self.downscale_x as u32);
        let index_y = indexed_location.index() / (self.downscale_x as u32);

        self.habitat
            .map_indexed_location_to_u64_injective(&IndexedLocation::new(
                Location::new(
                    indexed_location.location().x() + index_x,
                    indexed_location.location().y() + index_y,
                ),
                0,
            ))
    }

    #[must_use]
    fn iter_habitable_locations(&self) -> Self::LocationIterator<'_> {
        // TODO: optimise
        let width = unsafe {
            OffByOneU32::new_unchecked(OffByOneU32::max().get() / (self.downscale_x as u64))
        };
        let height = unsafe {
            OffByOneU32::new_unchecked(OffByOneU32::max().get() / (self.downscale_y as u64))
        };

        LandscapeExtent::new(Location::new(0, 0), width, height)
            .iter()
            .map(|location| {
                Location::new(
                    location.x() * (self.downscale_x as u32),
                    location.y() * (self.downscale_y as u32),
                )
            })
    }
}

#[contract_trait]
impl<M: MathsCore, G: RngCore<M>> UniformlySampleableHabitat<M, G>
    for AlmostInfiniteDownscaledHabitat<M>
{
    #[must_use]
    #[inline]
    fn sample_habitable_indexed_location(&self, rng: &mut G) -> IndexedLocation {
        // TODO: optimise
        let location = self.habitat.sample_habitable_indexed_location(rng);

        let index_x = location.location().x() % (self.downscale_x as u32);
        let index_y = location.location().y() % (self.downscale_y as u32);

        IndexedLocation::new(
            Location::new(
                location.location().x() - index_x,
                location.location().y() - index_y,
            ),
            index_y * (self.downscale_x as u32) + index_x,
        )
    }
}

impl serde::Serialize for Log2U16 {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        if serializer.is_human_readable() {
            serializer.collect_str(&format_args!("1B{}", self.log2()))
        } else {
            serializer.serialize_u32((*self) as u32)
        }
    }
}

impl<'de> serde::Deserialize<'de> for Log2U16 {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct Log2U16Visitor;

        impl<'de> serde::de::Visitor<'de> for Log2U16Visitor {
            type Value = Log2U16;

            fn expecting(&self, fmt: &mut alloc::fmt::Formatter) -> alloc::fmt::Result {
                fmt.write_str("an integer in 2^{0..=15} or its base-two scientific notation string")
            }

            fn visit_u64<E: serde::de::Error>(self, v: u64) -> Result<Self::Value, E> {
                match v {
                    const { Log2U16::Shl0 as u64 } => Ok(Log2U16::Shl0),
                    const { Log2U16::Shl1 as u64 } => Ok(Log2U16::Shl1),
                    const { Log2U16::Shl2 as u64 } => Ok(Log2U16::Shl2),
                    const { Log2U16::Shl3 as u64 } => Ok(Log2U16::Shl3),
                    const { Log2U16::Shl4 as u64 } => Ok(Log2U16::Shl4),
                    const { Log2U16::Shl5 as u64 } => Ok(Log2U16::Shl5),
                    const { Log2U16::Shl6 as u64 } => Ok(Log2U16::Shl6),
                    const { Log2U16::Shl7 as u64 } => Ok(Log2U16::Shl7),
                    const { Log2U16::Shl8 as u64 } => Ok(Log2U16::Shl8),
                    const { Log2U16::Shl9 as u64 } => Ok(Log2U16::Shl9),
                    const { Log2U16::Shl10 as u64 } => Ok(Log2U16::Shl10),
                    const { Log2U16::Shl11 as u64 } => Ok(Log2U16::Shl11),
                    const { Log2U16::Shl12 as u64 } => Ok(Log2U16::Shl12),
                    const { Log2U16::Shl13 as u64 } => Ok(Log2U16::Shl13),
                    const { Log2U16::Shl14 as u64 } => Ok(Log2U16::Shl14),
                    const { Log2U16::Shl15 as u64 } => Ok(Log2U16::Shl15),
                    v => Err(serde::de::Error::invalid_value(
                        serde::de::Unexpected::Unsigned(v),
                        &self,
                    )),
                }
            }

            fn visit_str<E: serde::de::Error>(self, v: &str) -> Result<Self::Value, E> {
                let Some(exp) = v.strip_prefix("1B") else {
                    return Err(serde::de::Error::invalid_value(
                        serde::de::Unexpected::Str(v),
                        &self,
                    ));
                };

                let Ok(exp) = exp.parse::<usize>() else {
                    return Err(serde::de::Error::invalid_value(
                        serde::de::Unexpected::Str(v),
                        &self,
                    ));
                };

                let Some(v) = [
                    Log2U16::Shl0,
                    Log2U16::Shl1,
                    Log2U16::Shl2,
                    Log2U16::Shl3,
                    Log2U16::Shl4,
                    Log2U16::Shl5,
                    Log2U16::Shl6,
                    Log2U16::Shl7,
                    Log2U16::Shl8,
                    Log2U16::Shl9,
                    Log2U16::Shl10,
                    Log2U16::Shl11,
                    Log2U16::Shl12,
                    Log2U16::Shl13,
                    Log2U16::Shl14,
                    Log2U16::Shl15,
                ]
                .get(exp) else {
                    return Err(serde::de::Error::invalid_value(
                        serde::de::Unexpected::Str(v),
                        &self,
                    ));
                };

                Ok(*v)
            }
        }

        if deserializer.is_human_readable() {
            deserializer.deserialize_str(Log2U16Visitor)
        } else {
            deserializer.deserialize_u32(Log2U16Visitor)
        }
    }
}
