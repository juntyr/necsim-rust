#![deny(clippy::pedantic)]

#[macro_use]
extern crate rustacuda;

use anyhow::Result;
use array2d::Array2D;

use rustacuda::context::Context as CudaContext;
use rustacuda::prelude::*;

use necsim_impls_std::cogs::habitat::in_memory::InMemoryHabitatBuilder;

use std::ffi::CString;

macro_rules! with_cuda {
    ($init:expr => |$var:ident: $r#type:ty| $inner:block) => {
        let $var = $init;

        $inner

        if let Err((_err, val)) = <$r#type>::drop($var) {
            //eprintln!("{:?}", err);
            core::mem::forget(val);
        }
    };
    ($init:expr => |mut $var:ident: $r#type:ty| $inner:block) => {
        let mut $var = $init;

        $inner

        if let Err((_err, val)) = <$r#type>::drop($var) {
            //eprintln!("{:?}", err);
            core::mem::forget(val);
        }
    };
}

fn main() -> Result<()> {
    let module_data = CString::new(include_str!(env!("KERNEL_PTX_PATH"))).unwrap();

    // Initialize the CUDA API
    rustacuda::init(CudaFlags::empty())?;

    // Get the first device
    let device = Device::get_device(0)?;

    // Create a context associated to this device
    with_cuda!(CudaContext::create_and_push(ContextFlags::MAP_HOST | ContextFlags::SCHED_AUTO, device)? => |context: CudaContext| {
    // Load the module containing the function we want to call
    with_cuda!(Module::load_from_string(&module_data)? => |module: Module| {
    // Create a stream to submit work to
    with_cuda!(Stream::new(StreamFlags::NON_BLOCKING, None)? => |stream: Stream| {

        let habitat_vec = vec![
            0, 1, 2, 3, 4, 5, 6, 7, 8,
            8, 7, 6, 5, 4, 3, 2, 1, 0,
            0, 1, 2, 3, 4, 5, 6, 7, 8
        ];

        let habitat_arr = Array2D::from_row_major(&habitat_vec, 3, 9);

        let habitat = InMemoryHabitatBuilder::from_array2d(&habitat_arr);

        if let Err(err) = InMemoryHabitatBuilder::lend_to_cuda(&habitat, |habitat| {
            // Launching kernels is unsafe since Rust can't enforce safety - think of kernel launches
            // as a foreign-function call. In this case, it is - this kernel is written in CUDA C.
            unsafe {
                launch!(module.test<<<1, 27, 0, stream>>>(
                    habitat
                ))?;
            }

            stream.synchronize()

        }) {
            eprintln!("Running kernel failed with {:#?}!", err);
        }

    });});});

    Ok(())
}
