use std::{
    ffi::{CStr, CString},
    os::raw::c_char,
};

use necsim_core::cogs::{
    ActiveLineageSampler, CoalescenceSampler, DispersalSampler, EventSampler,
    HabitatToU64Injection, IncoherentLineageStore, LineageReference, PrimeableRng,
};
use rustacuda_core::DeviceCopy;

use rust_cuda::common::RustToCuda;

extern "C" {
    fn get_ptx_cstr_for_specialisation(specialisation: *const c_char) -> *const c_char;
}

pub fn get_ptx_cstr<
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
>() -> &'static CStr {
    fn type_name_of<T>(_: T) -> CString {
        CString::new(std::any::type_name::<T>()).unwrap()
    }

    let type_name_cstring =
        type_name_of(get_ptx_cstr::<H, G, D, R, S, C, E, A, REPORT_SPECIATION, REPORT_DISPERSAL>);

    let ptx_c_chars = unsafe { get_ptx_cstr_for_specialisation(type_name_cstring.as_ptr()) };

    unsafe { CStr::from_ptr(ptx_c_chars as *const i8) }
}
