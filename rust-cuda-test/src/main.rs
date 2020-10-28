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

        InMemoryHabitatBuilder::lend_to_cuda(&habitat, |habitat| {

        // Allocate space on the device and copy numbers to it.
        //with_cuda!(DeviceBuffer::from_slice(&[10.0f32, 20.0f32])? => |mut x: DeviceBuffer<f32>| {
        //with_cuda!(DeviceBuffer::from_slice(&[20.0f32, -1.0f32])? => |mut y: DeviceBuffer<f32>| {
        //with_cuda!(DeviceBuffer::from_slice(&[0.0f32, 0.0f32])? => |mut result: DeviceBuffer<f32>| {

            // Launching kernels is unsafe since Rust can't enforce safety - think of kernel launches
            // as a foreign-function call. In this case, it is - this kernel is written in CUDA C.
            unsafe {
                // Launch the `add` function with one block containing one thread on the given stream.
                //launch!(module.add<<<1, 2, 0, stream>>>(
                launch!(module.test<<<1, 2, 0, stream>>>(
                    habitat
                    //x.as_device_ptr(),
                    //y.as_device_ptr(),
                    //result.as_device_ptr(),
                    //result.len() // Length (usize type MUST match)
                ))?;
            }

            stream.synchronize()

            // The kernel launch is asynchronous, so we wait for the kernel to finish executing
            //if let Err(err) = stream.synchronize() {
            //    eprintln!("Synchronisation failed with {:#?}", err);
            //} else {
                // Copy the result back to the host
                //let mut result_host = [0.0f32, 0.0f32];
                //result.copy_to(&mut result_host)?;

                //println!("Sum is {:?}", result_host);
            //}

        })?;
        //});});});

    });});});

    Ok(())
}
