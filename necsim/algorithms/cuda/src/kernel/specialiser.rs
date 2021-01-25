use std::{
    ffi::{CStr, CString},
    os::raw::c_char,
};

use necsim_core::cogs::{
    CoalescenceSampler, DispersalSampler, EmigrationExit, Habitat, ImmigrationEntry,
    LineageReference, LineageStore, MinSpeciationTrackingEventSampler, PrimeableRng,
    SingularActiveLineageSampler, SpeciationProbability,
};
use rustacuda_core::DeviceCopy;

use rust_cuda::common::RustToCuda;

extern "C" {
    fn get_ptx_cstr_for_specialisation(specialisation: *const c_char) -> *const c_char;
}

pub fn get_ptx_cstr<
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
>() -> &'static CStr {
    let type_name_cstring = type_name_of(
        get_ptx_cstr::<H, G, N, D, R, S, X, C, E, I, A, REPORT_SPECIATION, REPORT_DISPERSAL>,
    );

    let ptx_c_chars = unsafe { get_ptx_cstr_for_specialisation(type_name_cstring.as_ptr()) };

    unsafe { CStr::from_ptr(ptx_c_chars as *const i8) }
}

fn type_name_of<T>(_: T) -> CString {
    CString::new(std::any::type_name::<T>()).unwrap()
}
