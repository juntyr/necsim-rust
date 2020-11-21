use std::{ffi::CString, marker::PhantomData, ops::Deref};

use anyhow::Result;

use necsim_core::cogs::{
    CoalescenceSampler, DispersalSampler, EventSampler, HabitatToU64Injection,
    IncoherentLineageStore, LineageReference, PrimeableRng, SingularActiveLineageSampler,
};

use rustacuda::module::Module;
use rustacuda_core::DeviceCopy;

use rust_cuda::{common::RustToCuda, host::CudaDropWrapper};

use crate::info;

use super::{specialiser, SimulationKernel};

impl<
        'k,
        H: HabitatToU64Injection + RustToCuda,
        G: PrimeableRng<H> + RustToCuda,
        D: DispersalSampler<H, G> + RustToCuda,
        R: LineageReference<H> + DeviceCopy,
        S: IncoherentLineageStore<H, R> + RustToCuda,
        C: CoalescenceSampler<H, G, R, S> + RustToCuda,
        E: EventSampler<H, G, D, R, S, C> + RustToCuda,
        A: SingularActiveLineageSampler<H, G, D, R, S, C, E> + RustToCuda,
        const REPORT_SPECIATION: bool,
        const REPORT_DISPERSAL: bool,
    > SimulationKernel<'k, H, G, D, R, S, C, E, A, REPORT_SPECIATION, REPORT_DISPERSAL>
{
    pub fn with_kernel<
        Q,
        F: FnOnce(
            &SimulationKernel<H, G, D, R, S, C, E, A, REPORT_SPECIATION, REPORT_DISPERSAL>,
        ) -> Result<Q>,
    >(
        inner: F,
    ) -> Result<Q> {
        // Load the module containing the kernel function
        let module = CudaDropWrapper::from(Module::load_from_string(specialiser::get_ptx_cstr::<
            H,
            G,
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

        info::print_kernel_function_attributes(&entry_point);

        let kernel = SimulationKernel {
            module: &module,
            entry_point: &entry_point,
            marker: PhantomData::<(H, G, D, R, S, C, E, A)>,
        };

        inner(&kernel)
    }
}

impl<
        'k,
        H: HabitatToU64Injection + RustToCuda,
        G: PrimeableRng<H> + RustToCuda,
        D: DispersalSampler<H, G> + RustToCuda,
        R: LineageReference<H> + DeviceCopy,
        S: IncoherentLineageStore<H, R> + RustToCuda,
        C: CoalescenceSampler<H, G, R, S> + RustToCuda,
        E: EventSampler<H, G, D, R, S, C> + RustToCuda,
        A: SingularActiveLineageSampler<H, G, D, R, S, C, E> + RustToCuda,
        const REPORT_SPECIATION: bool,
        const REPORT_DISPERSAL: bool,
    > Deref for SimulationKernel<'k, H, G, D, R, S, C, E, A, REPORT_SPECIATION, REPORT_DISPERSAL>
{
    type Target = Module;

    fn deref(&self) -> &Self::Target {
        self.module
    }
}
