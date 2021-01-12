use std::marker::PhantomData;

use necsim_core::cogs::{
    CoalescenceSampler, DispersalSampler, EmigrationExit, Habitat, IncoherentLineageStore,
    LineageReference, MinSpeciationTrackingEventSampler, PrimeableRng,
    SingularActiveLineageSampler, SpeciationProbability,
};

use rustacuda::{function::Function, module::Module};
use rustacuda_core::DeviceCopy;

use rust_cuda::common::RustToCuda;

mod r#impl;
mod launch;
mod specialiser;

#[allow(clippy::module_name_repetitions)]
#[allow(clippy::type_complexity)]
pub struct SimulationKernel<
    'k,
    H: Habitat + RustToCuda,
    G: PrimeableRng<H> + RustToCuda,
    N: SpeciationProbability<H> + RustToCuda,
    D: DispersalSampler<H, G> + RustToCuda,
    R: LineageReference<H> + DeviceCopy,
    S: IncoherentLineageStore<H, R> + RustToCuda,
    X: EmigrationExit<H, G, N, D, R, S> + RustToCuda,
    C: CoalescenceSampler<H, G, R, S> + RustToCuda,
    E: MinSpeciationTrackingEventSampler<H, G, N, D, R, S, X, C> + RustToCuda,
    A: SingularActiveLineageSampler<H, G, N, D, R, S, X, C, E> + RustToCuda,
    const REPORT_SPECIATION: bool,
    const REPORT_DISPERSAL: bool,
> {
    module: &'k Module,
    entry_point: &'k Function<'k>,
    marker: PhantomData<(H, G, N, D, R, S, X, C, E, A)>,
}
