use std::{ffi::CString, marker::PhantomData, ops::Deref};

use anyhow::Result;

use necsim_core::cogs::{
    CoalescenceSampler, DispersalSampler, EmigrationExit, Habitat, ImmigrationEntry,
    LineageReference, LineageStore, MinSpeciationTrackingEventSampler, PrimeableRng,
    SingularActiveLineageSampler, SpeciationProbability,
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
        S: LineageStore<H, R> + RustToCuda,
        X: EmigrationExit<H, G, N, D, R, S> + RustToCuda,
        C: CoalescenceSampler<H, R, S> + RustToCuda,
        E: MinSpeciationTrackingEventSampler<H, G, N, D, R, S, X, C> + RustToCuda,
        I: ImmigrationEntry + RustToCuda,
        A: SingularActiveLineageSampler<H, G, N, D, R, S, X, C, E, I> + RustToCuda,
        const REPORT_SPECIATION: bool,
        const REPORT_DISPERSAL: bool,
    > SimulationKernel<'k, H, G, N, D, R, S, X, C, E, I, A, REPORT_SPECIATION, REPORT_DISPERSAL>
{
    pub fn with_kernel<
        Q,
        F: FnOnce(
            &SimulationKernel<H, G, N, D, R, S, X, C, E, I, A, REPORT_SPECIATION, REPORT_DISPERSAL>,
        ) -> Result<Q>,
    >(
        inner: F,
    ) -> Result<Q> {
        // Load the module PTX &CStr containing the kernel function
        let ptx_cstr = specialiser::get_ptx_cstr::<
            H,
            G,
            N,
            D,
            R,
            S,
            X,
            C,
            E,
            I,
            A,
            REPORT_SPECIATION,
            REPORT_DISPERSAL,
        >();

        let mut jit = ptx_jit::PtxJIT::new(ptx_cstr);

        println!("{:?}", jit.with_arguments(None));
        println!(
            "{:?}",
            jit.with_arguments(Some(
                vec![vec![0x1, 0x2, 0x3, 0x4].into_boxed_slice()].into_boxed_slice()
            ))
        );

        // JIT compile the module
        let module = CudaDropWrapper::from(Module::load_from_string(ptx_cstr)?);

        // Load the kernel function from the module
        let entry_point = module.get_function(&CString::new("simulate").unwrap())?;

        let kernel = SimulationKernel {
            module: &module,
            entry_point: &entry_point,
            marker: PhantomData::<(H, G, N, D, R, S, X, C, E, I, A)>,
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
        S: LineageStore<H, R> + RustToCuda,
        X: EmigrationExit<H, G, N, D, R, S> + RustToCuda,
        C: CoalescenceSampler<H, R, S> + RustToCuda,
        E: MinSpeciationTrackingEventSampler<H, G, N, D, R, S, X, C> + RustToCuda,
        I: ImmigrationEntry + RustToCuda,
        A: SingularActiveLineageSampler<H, G, N, D, R, S, X, C, E, I> + RustToCuda,
        const REPORT_SPECIATION: bool,
        const REPORT_DISPERSAL: bool,
    > Deref
    for SimulationKernel<'k, H, G, N, D, R, S, X, C, E, I, A, REPORT_SPECIATION, REPORT_DISPERSAL>
{
    type Target = Module;

    fn deref(&self) -> &Self::Target {
        self.module
    }
}
