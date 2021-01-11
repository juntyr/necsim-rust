use std::{ffi::CString, marker::PhantomData, ops::Deref};

use anyhow::Result;

use necsim_core::cogs::{
    CoalescenceSampler, DispersalSampler, Habitat, IncoherentLineageStore, LineageReference,
    MinSpeciationTrackingEventSampler, PrimeableRng, SingularActiveLineageSampler,
    SpeciationProbability,
};

use rustacuda::{function::Function, module::Module};
use rustacuda_core::DeviceCopy;

use rust_cuda::{common::RustToCuda, host::CudaDropWrapper};

use super::{specialiser, SimulationKernel};

impl<
        'k,
        H: Habitat + RustToCuda,
        G: PrimeableRng<H> + RustToCuda,
        N: SpeciationProbability<H> + RustToCuda,
        D: DispersalSampler<H, G> + RustToCuda,
        R: LineageReference<H> + DeviceCopy,
        S: IncoherentLineageStore<H, R> + RustToCuda,
        C: CoalescenceSampler<H, G, R, S> + RustToCuda,
        E: MinSpeciationTrackingEventSampler<H, G, N, D, R, S, C> + RustToCuda,
        A: SingularActiveLineageSampler<H, G, N, D, R, S, C, E> + RustToCuda,
        const REPORT_SPECIATION: bool,
        const REPORT_DISPERSAL: bool,
    > SimulationKernel<'k, H, G, N, D, R, S, C, E, A, REPORT_SPECIATION, REPORT_DISPERSAL>
{
    pub fn with_kernel<
        Q,
        F: FnOnce(
            &SimulationKernel<H, G, N, D, R, S, C, E, A, REPORT_SPECIATION, REPORT_DISPERSAL>,
        ) -> Result<Q>,
    >(
        inner: F,
    ) -> Result<Q> {
        // Load the module containing the kernel function
        let module = CudaDropWrapper::from(Module::load_from_string(specialiser::get_ptx_cstr::<
            H,
            G,
            N,
            D,
            R,
            S,
            C,
            E,
            A,
            REPORT_SPECIATION,
            REPORT_DISPERSAL,
        >())?);

        // Load the kernel function from the module
        let entry_point = module.get_function(&CString::new("simulate").unwrap())?;

        let kernel = SimulationKernel {
            module: &module,
            entry_point: &entry_point,
            marker: PhantomData::<(H, G, N, D, R, S, C, E, A)>,
        };

        inner(&kernel)
    }

    pub fn function(&self) -> &Function {
        &self.entry_point
    }
}

impl<
        'k,
        H: Habitat + RustToCuda,
        G: PrimeableRng<H> + RustToCuda,
        N: SpeciationProbability<H> + RustToCuda,
        D: DispersalSampler<H, G> + RustToCuda,
        R: LineageReference<H> + DeviceCopy,
        S: IncoherentLineageStore<H, R> + RustToCuda,
        C: CoalescenceSampler<H, G, R, S> + RustToCuda,
        E: MinSpeciationTrackingEventSampler<H, G, N, D, R, S, C> + RustToCuda,
        A: SingularActiveLineageSampler<H, G, N, D, R, S, C, E> + RustToCuda,
        const REPORT_SPECIATION: bool,
        const REPORT_DISPERSAL: bool,
    > Deref
    for SimulationKernel<'k, H, G, N, D, R, S, C, E, A, REPORT_SPECIATION, REPORT_DISPERSAL>
{
    type Target = Module;

    fn deref(&self) -> &Self::Target {
        self.module
    }
}
