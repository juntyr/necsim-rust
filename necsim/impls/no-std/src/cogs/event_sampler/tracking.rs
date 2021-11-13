use core::hash::{Hash, Hasher};

use necsim_core_bond::{ClosedOpenUnitF64, PositiveF64};

use necsim_core::{
    cogs::{
        CoalescenceSampler, DispersalSampler, EmigrationExit, EventSampler, Habitat,
        LineageReference, LineageStore, MathsCore, RngCore, SpeciationProbability, TurnoverRate,
    },
    landscape::IndexedLocation,
};

pub trait MinSpeciationTrackingEventSampler<
    M: MathsCore,
    H: Habitat<M>,
    G: RngCore<M>,
    R: LineageReference<M, H>,
    S: LineageStore<M, H, R>,
    X: EmigrationExit<M, H, G, R, S>,
    D: DispersalSampler<M, H, G>,
    C: CoalescenceSampler<M, H, R, S>,
    T: TurnoverRate<M, H>,
    N: SpeciationProbability<M, H>,
>: EventSampler<M, H, G, R, S, X, D, C, T, N>
{
    fn replace_min_speciation(&mut self, new: Option<SpeciationSample>)
        -> Option<SpeciationSample>;
}

#[derive(Clone, Debug, TypeLayout)]
#[repr(C)]
pub struct SpeciationSample {
    speciation_sample: ClosedOpenUnitF64,
    sample_time: PositiveF64,
    sample_location: IndexedLocation,
}

#[allow(dead_code)]
const EXCESSIVE_OPTION_SPECIATION_SAMPLE_ERROR: [(); 1 - {
    const ASSERT: bool = core::mem::size_of::<Option<SpeciationSample>>()
        == core::mem::size_of::<SpeciationSample>();
    ASSERT
} as usize] = [];

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
                    sample_location: sample_location.clone(),
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
