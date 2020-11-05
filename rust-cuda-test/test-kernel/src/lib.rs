#![cfg(target_os = "cuda")]
#![deny(clippy::pedantic)]
#![no_std]
#![feature(abi_ptx)]
#![feature(doc_cfg)]
#![feature(link_llvm_intrinsics)]
#![feature(core_intrinsics)]
#![feature(alloc_error_handler)]

extern crate alloc;

use rust_cuda::{
    device::{nvptx, utils},
    println,
};

#[global_allocator]
static _GLOBAL_ALLOCATOR: utils::PTXAllocator = utils::PTXAllocator;

#[panic_handler]
fn panic(_info: &::core::panic::PanicInfo) -> ! {
    unsafe { nvptx::trap() }
}

#[alloc_error_handler]
fn alloc_error_handler(_: core::alloc::Layout) -> ! {
    unsafe { nvptx::trap() }
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

use necsim_core::cogs::Habitat;
use necsim_core::landscape::Location;
use necsim_impls_no_std::cogs::habitat::in_memory::InMemoryHabitat;
use rust_cuda::device::BorrowFromRust;

#[no_mangle]
/// # Safety
/// This CUDA kernel is unsafe as it is called with raw pointers
pub unsafe extern "ptx-kernel" fn test_habitat(
    habitat_ptr: *const <InMemoryHabitat as rust_cuda::common::RustToCuda>::CudaRepresentation,
) {
    InMemoryHabitat::with_borrow_from_rust(habitat_ptr, |habitat| {
        #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
        let thread_index = utils::index() as u32;

        let location = Location::new(
            thread_index % habitat.get_extent().width(),
            thread_index / habitat.get_extent().width(),
        );

        println!(
            "Habitat with extent {:?}, total habitat {}, and habitat {} at {:?}.",
            habitat.get_extent(),
            habitat.get_total_habitat(),
            habitat.get_habitat_at_location(&location),
            location,
        );
    })
}
