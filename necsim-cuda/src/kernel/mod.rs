use std::marker::PhantomData;

use necsim_core::cogs::{
    ActiveLineageSampler, CoalescenceSampler, DispersalSampler, EventSampler,
    HabitatToU64Injection, IncoherentLineageStore, LineageReference, PrimeableRng,
};

use rustacuda::{function::Function, module::Module};
use rustacuda_core::DeviceCopy;

use rust_cuda::common::RustToCuda;

mod r#impl;
mod launch;
mod specialiser;

pub struct SimulationKernel<
    'k,
    H: HabitatToU64Injection + RustToCuda,
    G: PrimeableRng<H> + RustToCuda,
    D: DispersalSampler<H, G> + RustToCuda,
    R: LineageReference<H> + DeviceCopy,
    S: IncoherentLineageStore<H, R> + RustToCuda,
    C: CoalescenceSampler<H, G, R, S> + RustToCuda,
    E: EventSampler<H, G, D, R, S, C> + RustToCuda,
    A: ActiveLineageSampler<H, G, D, R, S, C, E> + RustToCuda,
    const REPORT_SPECIATION: bool,
    const REPORT_DISPERSAL: bool,
> {
    module: &'k Module,
    entry_point: &'k Function<'k>,
    marker: PhantomData<(H, G, D, R, S, C, E, A)>,
}
