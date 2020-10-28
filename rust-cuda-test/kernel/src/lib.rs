#![cfg(target_os = "cuda")]
#![no_std]
#![feature(abi_ptx)]
#![feature(doc_cfg)]
#![feature(link_llvm_intrinsics)]
#![feature(core_intrinsics)]
#![feature(alloc_error_handler)]

extern crate alloc;

pub mod arch;

#[macro_use]
pub mod utils;

#[global_allocator]
static _GLOBAL_ALLOCATOR: utils::PTXAllocator = utils::PTXAllocator;

#[panic_handler]
fn panic(_info: &::core::panic::PanicInfo) -> ! {
    unsafe { arch::nvptx::trap() }
}

#[alloc_error_handler]
fn alloc_error_handler(_: core::alloc::Layout) -> ! {
    unsafe { arch::nvptx::trap() }
}

struct F32(f32);
struct F64(f64);

impl core::fmt::Debug for F32 {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", ryu::Buffer::new().format(self.0))
    }
}

impl core::fmt::Debug for F64 {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", ryu::Buffer::new().format(self.0))
    }
}

#[no_mangle]
pub unsafe extern "ptx-kernel" fn add(a: *const f32, b: *const f32, c: *mut f32, n: usize) {
    let a: &[f32] = core::slice::from_raw_parts(a, n);
    let b: &[f32] = core::slice::from_raw_parts(b, n);

    let c: &mut [f32] = core::slice::from_raw_parts_mut(c, n);

    println!(
        "a={:?}",
        core::slice::from_raw_parts(a.as_ptr() as *const F32, n)
    );
    println!(
        "b={:?}",
        core::slice::from_raw_parts(b.as_ptr() as *const F32, n)
    );

    add_impl(&a, &b, c);

    println!(
        "c={:?}",
        core::slice::from_raw_parts(c.as_ptr() as *const F32, n)
    );
}

pub fn add_impl(a: &[f32], b: &[f32], c: &mut [f32]) {
    let i = utils::index() as usize;

    if let (Some(a), Some(b), Some(c)) = (a.get(i), b.get(i), c.get_mut(i)) {
        *c = a + b;
    }
}

use necsim_core::cogs::Habitat;
use necsim_core::landscape::Location;
use necsim_impls_no_std::cogs::habitat::in_memory::cuda::InMemoryHabitatCuda;
use necsim_impls_no_std::cogs::habitat::in_memory::InMemoryHabitat;

#[no_mangle]
pub unsafe extern "ptx-kernel" fn test(habitat: *const InMemoryHabitatCuda) {
    InMemoryHabitat::with_ref(habitat, |habitat| {
        println!("Extent: {:#?}", habitat.get_extent());
        println!("Total habitat: {}", habitat.get_total_habitat());
        println!(
            "Habitat at (2,8): {}",
            habitat.get_habitat_at_location(&Location::new(2, 8))
        );
    })
}
