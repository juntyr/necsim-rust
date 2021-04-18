use std::{
    ffi::{CStr, CString},
    os::raw::c_char,
};

use necsim_core::{
    cogs::{
        CoalescenceSampler, DispersalSampler, EmigrationExit, Habitat, ImmigrationEntry,
        LineageReference, LineageStore, MinSpeciationTrackingEventSampler, PrimeableRng,
        SingularActiveLineageSampler, SpeciationProbability, TurnoverRate,
    },
    reporter::boolean::Boolean,
};

use rustacuda_core::DeviceCopy;

use rust_cuda::common::RustToCuda;

extern "C" {
    fn get_ptx_cstr_for_specialisation(specialisation: *const c_char) -> *const c_char;
}

pub fn get_ptx_cstr<
    H: Habitat + RustToCuda,
    G: PrimeableRng + RustToCuda,
    R: LineageReference<H> + DeviceCopy,
    S: LineageStore<H, R> + RustToCuda,
    X: EmigrationExit<H, G, R, S> + RustToCuda,
    D: DispersalSampler<H, G> + RustToCuda,
    C: CoalescenceSampler<H, R, S> + RustToCuda,
    T: TurnoverRate<H> + RustToCuda,
    N: SpeciationProbability<H> + RustToCuda,
    E: MinSpeciationTrackingEventSampler<H, G, R, S, X, D, C, T, N> + RustToCuda,
    I: ImmigrationEntry + RustToCuda,
    A: SingularActiveLineageSampler<H, G, R, S, X, D, C, T, N, E, I> + RustToCuda,
    ReportSpeciation: Boolean,
    ReportDispersal: Boolean,
>() -> &'static CStr {
    let type_name_cstring = type_name_of(
        get_ptx_cstr::<H, G, R, S, X, D, C, T, N, E, I, A, ReportSpeciation, ReportDispersal>,
    );

    let ptx_c_chars = unsafe { get_ptx_cstr_for_specialisation(type_name_cstring.as_ptr()) };

    unsafe { CStr::from_ptr(ptx_c_chars.cast::<i8>()) }
}

fn type_name_of<T>(_: T) -> CString {
    CString::new(std::any::type_name::<T>()).unwrap()
}
