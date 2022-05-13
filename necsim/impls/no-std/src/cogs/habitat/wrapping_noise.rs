use alloc::boxed::Box;
use core::{f64::consts::PI, fmt};
use necsim_core_bond::OffByOneU64;

use opensimplex_noise_rs::{OpenSimplexNoise, PermTable as OpenSimplexTable};

use necsim_core::{
    cogs::{Backup, Habitat, MathsCore, RngCore, UniformlySampleableHabitat},
    landscape::{IndexedLocation, LandscapeExtent, Location},
};

use crate::cogs::{
    habitat::almost_infinite::AlmostInfiniteHabitat,
    lineage_store::coherent::globally::singleton_demes::SingletonDemesHabitat,
};

#[allow(clippy::module_name_repetitions)]
#[cfg_attr(feature = "cuda", derive(rust_cuda::common::LendRustToCuda))]
#[cfg_attr(feature = "cuda", cuda(free = "M"))]
pub struct WrappingNoiseHabitat<M: MathsCore> {
    #[cfg_attr(feature = "cuda", cuda(embed))]
    inner: AlmostInfiniteHabitat<M>,
    threshold: f64,
    #[cfg_attr(feature = "cuda", cuda(embed))]
    noise: Box<OpenSimplexTable>,
}

impl<M: MathsCore> fmt::Debug for WrappingNoiseHabitat<M> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_struct(stringify!(WrappingNoiseHabitat))
            .field("inner", &self.inner)
            .field("threshold", &self.threshold)
            .finish()
    }
}

impl<M: MathsCore> WrappingNoiseHabitat<M> {
    #[must_use]
    pub fn new(seed: i64, threshold: f64) -> Self {
        let noise = Box::new(OpenSimplexNoise::new(Some(seed)));

        Self {
            inner: AlmostInfiniteHabitat::default(),
            threshold,
            noise: unsafe { core::mem::transmute(noise) },
        }
    }

    pub(crate) fn get_inner(&self) -> &AlmostInfiniteHabitat<M> {
        &self.inner
    }
}

impl<M: MathsCore> Default for WrappingNoiseHabitat<M> {
    fn default() -> Self {
        let noise = Box::new(OpenSimplexNoise::new(None));

        Self {
            inner: AlmostInfiniteHabitat::default(),
            threshold: 0.0_f64,
            noise: unsafe { core::mem::transmute(noise) },
        }
    }
}

#[contract_trait]
impl<M: MathsCore> Backup for WrappingNoiseHabitat<M> {
    unsafe fn backup_unchecked(&self) -> Self {
        Self {
            inner: self.inner.backup_unchecked(),
            threshold: self.threshold,
            noise: self.noise.clone(),
        }
    }
}

#[contract_trait]
impl<M: MathsCore> Habitat<M> for WrappingNoiseHabitat<M> {
    type LocationIterator<'a> = impl Iterator<Item = Location> + 'a;

    #[must_use]
    fn get_extent(&self) -> &LandscapeExtent {
        self.inner.get_extent()
    }

    #[must_use]
    fn get_total_habitat(&self) -> OffByOneU64 {
        // WARNING: just the upper bound
        self.inner.get_total_habitat()
    }

    #[must_use]
    fn get_habitat_at_location(&self, location: &Location) -> u32 {
        let (x, y, z, w) = location_to_wrapping_4d::<M>(location);

        let noise: &OpenSimplexNoise = unsafe { &*core::ptr::addr_of!(*self.noise).cast() };

        u32::from(noise.eval_4d(x, y, z, w) > self.threshold)
    }

    #[must_use]
    fn map_indexed_location_to_u64_injective(&self, indexed_location: &IndexedLocation) -> u64 {
        self.inner
            .map_indexed_location_to_u64_injective(indexed_location)
    }

    #[must_use]
    fn iter_habitable_locations(&self) -> Self::LocationIterator<'_> {
        self.get_extent()
            .iter()
            .filter(move |location| self.get_habitat_at_location(location) > 0)
    }
}

#[contract_trait]
impl<M: MathsCore, G: RngCore<M>> UniformlySampleableHabitat<M, G> for WrappingNoiseHabitat<M> {
    #[must_use]
    fn sample_habitable_indexed_location(&self, rng: &mut G) -> IndexedLocation {
        // Rejection sample until a habitable location is found
        let location = loop {
            let index = rng.sample_u64();

            let location = Location::new(
                (index & 0xFFFF_FFFF) as u32,
                ((index >> 32) & 0xFFFF_FFFF) as u32,
            );

            if self.get_habitat_at_location(&location) > 0 {
                break location;
            }
        };

        IndexedLocation::new(location, 0)
    }
}

impl<M: MathsCore> SingletonDemesHabitat<M> for WrappingNoiseHabitat<M> {}

const U32_MAX_AS_F64: f64 = (u32::MAX as f64) + 1.0_f64;

const X1: f64 = -1.0_f64;
const X2: f64 = 1.0_f64;
const Y1: f64 = -1.0_f64;
const Y2: f64 = 1.0_f64;

// Adapted from JTippetts' Seamless Noise article on gamedev.net:
//  https://www.gamedev.net/blog/33/entry-2138456-seamless-noise/
fn location_to_wrapping_4d<M: MathsCore>(location: &Location) -> (f64, f64, f64, f64) {
    let s = f64::from(location.x()) / U32_MAX_AS_F64;
    let t = f64::from(location.y()) / U32_MAX_AS_F64;

    let dx = X2 - X1;
    let dy = Y2 - Y1;

    let nx = X1 + M::cos(s * 2.0_f64 * PI) * dx / (2.0_f64 * PI);
    let ny = Y1 + M::cos(t * 2.0_f64 * PI) * dy / (2.0_f64 * PI);
    let nz = X1 + M::sin(s * 2.0_f64 * PI) * dx / (2.0_f64 * PI);
    let nw = Y1 + M::sin(t * 2.0_f64 * PI) * dy / (2.0_f64 * PI);

    (nx, ny, nz, nw)
}
