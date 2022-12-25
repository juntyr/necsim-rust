use alloc::boxed::Box;
use core::{f64::consts::PI, fmt, num::NonZeroUsize};
use necsim_core_bond::{ClosedUnitF64, OffByOneU64, OpenClosedUnitF64 as PositiveUnitF64};
use r#final::Final;

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
    coverage: ClosedUnitF64,
    threshold: f64,
    scale: PositiveUnitF64,
    persistence: PositiveUnitF64,
    octaves: NonZeroUsize,
    #[cfg_attr(feature = "cuda", cuda(embed))]
    noise: Final<Box<OpenSimplexTable>>,
}

impl<M: MathsCore> fmt::Debug for WrappingNoiseHabitat<M> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_struct(stringify!(WrappingNoiseHabitat))
            .field("inner", &self.inner)
            .field("coverage", &self.coverage)
            .field("scale", &self.scale)
            .field("persistence", &self.persistence)
            .field("octaves", &self.octaves)
            .finish()
    }
}

impl<M: MathsCore> WrappingNoiseHabitat<M> {
    #[must_use]
    pub fn new(
        seed: i64,
        coverage: ClosedUnitF64,
        scale: PositiveUnitF64,
        persistence: PositiveUnitF64,
        octaves: NonZeroUsize,
    ) -> Self {
        const THRESHOLD_QUALITY_LOG2: u32 = 8;

        let noise = Box::new(OpenSimplexNoise::new(Some(seed)));

        // Emperically determine a threshold to uniformly sample habitat
        //  from the generated Simplex Noise
        let mut samples = alloc::vec::Vec::new();

        for y in 0..(1 << THRESHOLD_QUALITY_LOG2) {
            for x in 0..(1 << THRESHOLD_QUALITY_LOG2) {
                samples.push(sum_noise_octaves::<M>(
                    &noise,
                    &Location::new(
                        x << (32 - THRESHOLD_QUALITY_LOG2),
                        y << (32 - THRESHOLD_QUALITY_LOG2),
                    ),
                    persistence,
                    scale,
                    octaves,
                ));
            }
        }

        samples.sort_unstable_by(f64::total_cmp);

        #[allow(clippy::cast_possible_truncation)]
        #[allow(clippy::cast_sign_loss)]
        #[allow(clippy::cast_precision_loss)]
        let threshold = samples
            [(M::floor((samples.len() as f64) * coverage.get()) as usize).min(samples.len() - 1)];

        Self {
            inner: AlmostInfiniteHabitat::default(),
            coverage,
            threshold,
            scale,
            persistence,
            octaves,
            noise: Final::new(unsafe { core::mem::transmute(noise) }),
        }
    }

    pub(crate) fn get_inner(&self) -> &AlmostInfiniteHabitat<M> {
        &self.inner
    }

    #[must_use]
    pub fn coverage(&self) -> ClosedUnitF64 {
        self.coverage
    }
}

impl<M: MathsCore> Default for WrappingNoiseHabitat<M> {
    fn default() -> Self {
        Self::new(
            0,
            ClosedUnitF64::half(),
            unsafe { PositiveUnitF64::new_unchecked(0.07_f64) },
            unsafe { PositiveUnitF64::new_unchecked(0.5_f64) },
            unsafe { NonZeroUsize::new_unchecked(16_usize) },
        )
    }
}

#[contract_trait]
impl<M: MathsCore> Backup for WrappingNoiseHabitat<M> {
    unsafe fn backup_unchecked(&self) -> Self {
        Self {
            inner: self.inner.backup_unchecked(),
            coverage: self.coverage,
            threshold: self.threshold,
            scale: self.scale,
            persistence: self.persistence,
            octaves: self.octaves,
            noise: Final::new(self.noise.clone()),
        }
    }
}

#[contract_trait]
impl<M: MathsCore> Habitat<M> for WrappingNoiseHabitat<M> {
    type LocationIterator<'a> = impl Iterator<Item = Location> + 'a;

    #[must_use]
    fn is_finite(&self) -> bool {
        self.coverage <= ClosedUnitF64::zero()
    }

    #[must_use]
    fn get_extent(&self) -> &LandscapeExtent {
        self.inner.get_extent()
    }

    #[must_use]
    fn get_total_habitat(&self) -> OffByOneU64 {
        // Note: This only gives a rough estimate
        self.inner.get_total_habitat() * self.coverage
    }

    #[must_use]
    fn get_habitat_at_location(&self, location: &Location) -> u32 {
        if self.coverage.get() <= 0.0_f64 {
            return 0;
        }

        if self.coverage.get() >= 1.0_f64 {
            return 1;
        }

        let noise: &OpenSimplexTable = &self.noise;
        let noise: &OpenSimplexNoise = unsafe { &*(noise as *const OpenSimplexTable).cast() };

        let noise =
            sum_noise_octaves::<M>(noise, location, self.persistence, self.scale, self.octaves);

        u32::from(noise <= self.threshold)
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

// Adapted from Christian Maher's article "Working with Simplex Noise"
// Licensed under CC BY 3.0
// Published at https://cmaher.github.io/posts/working-with-simplex-noise/
fn sum_noise_octaves<M: MathsCore>(
    noise: &OpenSimplexNoise,
    location: &Location,
    persistence: PositiveUnitF64,
    scale: PositiveUnitF64,
    octaves: NonZeroUsize,
) -> f64 {
    let mut max_amplitude = 0.0_f64;
    let mut amplitude = 1.0_f64;
    let mut frequency = scale.get();

    let mut result = 0.0_f64;

    for _ in 0..octaves.get() {
        let (x, y, z, w) = location_to_wrapping_4d::<M>(location, frequency);
        result += noise.eval_4d(x, y, z, w) * amplitude;
        max_amplitude += amplitude;
        amplitude *= persistence.get();
        frequency *= 2.0_f64;
    }

    result / max_amplitude
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

// Adapted from JTippetts' Seamless Noise article on gamedev.net:
//  https://www.gamedev.net/blog/33/entry-2138456-seamless-noise/
fn location_to_wrapping_4d<M: MathsCore>(location: &Location, scale: f64) -> (f64, f64, f64, f64) {
    let s = f64::from(location.x()) / U32_MAX_AS_F64;
    let t = f64::from(location.y()) / U32_MAX_AS_F64;

    let scale = scale * U32_MAX_AS_F64;

    let nx = M::cos(s * 2.0_f64 * PI) * scale;
    let ny = M::cos(t * 2.0_f64 * PI) * scale;
    let nz = M::sin(s * 2.0_f64 * PI) * scale;
    let nw = M::sin(t * 2.0_f64 * PI) * scale;

    (nx, ny, nz, nw)
}
