use anyhow::Result;

use rust_cuda::rustacuda::{
    context::{Context, CurrentContext, ResourceLimit},
    prelude::*,
};

use rust_cuda::host::CudaDropWrapper;

use crate::info;

#[allow(clippy::module_name_repetitions)]
pub fn with_initialised_cuda<O, F: FnOnce() -> Result<O>>(device: u32, inner: F) -> Result<O> {
    // Initialize the CUDA API
    rust_cuda::rustacuda::init(CudaFlags::empty())?;

    // Get the first device
    let device = Device::get_device(device)?;

    // Create a context associated to this device
    let context = CudaDropWrapper::from(Context::create_and_push(
        ContextFlags::MAP_HOST | ContextFlags::SCHED_AUTO,
        device,
    )?);

    CurrentContext::set_resource_limit(ResourceLimit::StackSize, 4096)?;

    info::print_context_resource_limits();

    let result = inner();

    // Explicit drop of the current CUDA context to explicitly end its scope
    std::mem::drop(context);

    result
}
