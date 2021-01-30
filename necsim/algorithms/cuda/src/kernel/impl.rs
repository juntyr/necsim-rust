use std::{cell::UnsafeCell, ffi::CString, marker::PhantomData, ops::Deref};

use anyhow::Result;

use necsim_core::cogs::{
    CoalescenceSampler, DispersalSampler, EmigrationExit, Habitat, ImmigrationEntry,
    LineageReference, LineageStore, MinSpeciationTrackingEventSampler, PrimeableRng,
    SingularActiveLineageSampler, SpeciationProbability,
};

use rustacuda::{function::Function, module::Module};
use rustacuda_core::DeviceCopy;

use ptx_jit::host::compiler::PtxJITCompiler;
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
    pub fn with_kernel<Q, F>(inner: F) -> Result<Q>
    where
        for<'s> F: FnOnce(
            &'s mut SimulationKernel<
                's,
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
            >,
        ) -> Result<Q>,
    {
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

        // Initialise the PTX JIT compiler with the original PTX source string
        let mut compiler = PtxJITCompiler::new(ptx_cstr);

        // Compile the CUDA module
        #[allow(unused_mut)]
        let mut module =
            UnsafeCell::new(CudaDropWrapper::from(Module::load_from_string(ptx_cstr)?));

        // Load the kernel function from the module
        let mut entry_point =
            unsafe { &*module.get() }.get_function(&CString::new("simulate").unwrap())?;

        // Safety: the mut `module` is only safe because:
        //  - `entry_point` is always dropped before `module` replaced
        //  - neither are mutably changed internally, only replaced
        let mut kernel = SimulationKernel {
            compiler: &mut compiler,
            module: unsafe { &mut *module.get() },
            entry_point: &mut entry_point,
            marker: PhantomData::<(H, G, N, D, R, S, X, C, E, I, A)>,
        };

        inner(&mut kernel)
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
