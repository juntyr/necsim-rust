use core::hash::{Hash, Hasher};

use necsim_core_bond::{ClosedOpenUnitF64, PositiveF64};

use necsim_core::{
    cogs::{
        CoalescenceSampler, DispersalSampler, EmigrationExit, EventSampler, Habitat, LineageStore,
        MathsCore, RngCore, SpeciationProbability, TurnoverRate,
    },
    landscape::IndexedLocation,
};

pub trait MinSpeciationTrackingEventSampler<
    M: MathsCore,
    H: Habitat<M>,
    G: RngCore<M>,
    S: LineageStore<M, H>,
    X: EmigrationExit<M, H, G, S>,
    D: DispersalSampler<M, H, G>,
    C: CoalescenceSampler<M, H, S>,
    T: TurnoverRate<M, H>,
    N: SpeciationProbability<M, H>,
>: EventSampler<M, H, G, S, X, D, C, T, N>
{
    fn replace_min_speciation(&mut self, new: Option<SpeciationSample>)
        -> Option<SpeciationSample>;
}

#[derive(Clone, Debug, TypeLayout)]
#[cfg_attr(feature = "cuda", derive(rust_cuda::lend::LendRustToCuda))]
#[repr(C)]
pub struct SpeciationSample {
    speciation_sample: ClosedOpenUnitF64,
    sample_time: PositiveF64,
    sample_location: IndexedLocation,
}

impl SpeciationSample {
    pub fn update_min(
        min_spec_sample: &mut Option<Self>,
        speciation_sample: ClosedOpenUnitF64,
        sample_time: PositiveF64,
        sample_location: &IndexedLocation,
    ) {
        match min_spec_sample {
            Some(min_spec_sample) if min_spec_sample.speciation_sample <= speciation_sample => (),
            _ => {
                *min_spec_sample = Some(Self {
                    speciation_sample,
                    sample_time,
                    sample_location: *sample_location,
                });
            },
        };
    }
}

impl PartialEq for SpeciationSample {
    fn eq(&self, other: &Self) -> bool {
        self.speciation_sample.eq(&other.speciation_sample)
            && self.sample_time.eq(&other.sample_time.get())
            && self.sample_location.eq(&other.sample_location)
    }
}

impl Eq for SpeciationSample {}

impl Hash for SpeciationSample {
    fn hash<S: Hasher>(&self, state: &mut S) {
        self.sample_location.hash(state);
        self.sample_time.hash(state);
        self.speciation_sample.hash(state);
    }
}
