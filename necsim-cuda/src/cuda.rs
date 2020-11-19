use std::ffi::{CStr, CString};

use anyhow::Result;

use rustacuda::{
    context::{Context, CurrentContext, ResourceLimit},
    function::Function,
    prelude::*,
};

use rust_cuda::host::CudaDropWrapper;

use crate::info;

#[allow(clippy::module_name_repetitions)]
pub fn with_cuda_kernel<O, F: FnOnce(&Stream, &Module, &Function) -> Result<O>>(
    module_data: &CStr,
    inner: F,
) -> Result<O> {
    // Initialize the CUDA API
    rustacuda::init(CudaFlags::empty())?;

    // Get the first device
    let device = Device::get_device(0)?;

    {
        // Create a context associated to this device
        let context = CudaDropWrapper::from(Context::create_and_push(
            ContextFlags::MAP_HOST | ContextFlags::SCHED_AUTO,
            device,
        )?);

        let result = {
            // Load the module containing the kernel function
            let module = CudaDropWrapper::from(Module::load_from_string(module_data)?);
            // Load the kernel function from the module
            let kernel = module.get_function(&CString::new("simulate").unwrap())?;
            // Create a stream to submit work to
            let stream = CudaDropWrapper::from(Stream::new(StreamFlags::NON_BLOCKING, None)?);

            CurrentContext::set_resource_limit(ResourceLimit::StackSize, 4096)?;

            info::print_context_resource_limits();
            info::print_kernel_function_attributes(&kernel);

            inner(&stream, &module, &kernel)
        };

        // Explicit drop of the current CUDA context to explicitly end its scope
        std::mem::drop(context);

        result
    }
}
