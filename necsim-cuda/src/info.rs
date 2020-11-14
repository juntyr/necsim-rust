use rustacuda::{
    context::{CurrentContext, ResourceLimit},
    function::{Function, FunctionAttribute},
};

pub fn print_context_resource_limits() {
    println!("{:=^80}", " Context Resource Limits ");

    println!(
        "StackSize: {:?}",
        CurrentContext::get_resource_limit(ResourceLimit::StackSize)
    );
    println!(
        "PrintfFifoSize: {:?}",
        CurrentContext::get_resource_limit(ResourceLimit::PrintfFifoSize)
    );
    println!(
        "MallocHeapSize: {:?}",
        CurrentContext::get_resource_limit(ResourceLimit::MallocHeapSize)
    );
    println!(
        "DeviceRuntimeSynchronizeDepth: {:?}",
        CurrentContext::get_resource_limit(ResourceLimit::DeviceRuntimeSynchronizeDepth)
    );
    println!(
        "DeviceRuntimePendingLaunchCount: {:?}",
        CurrentContext::get_resource_limit(ResourceLimit::DeviceRuntimePendingLaunchCount)
    );
    println!(
        "MaxL2FetchGranularity: {:?}",
        CurrentContext::get_resource_limit(ResourceLimit::MaxL2FetchGranularity)
    );

    println!("{:=^80}", "");
}

pub fn print_kernel_function_attributes(kernel: &Function) {
    println!("{:=^80}", " Kernel Function Attributes ");

    println!(
        "MaxThreadsPerBlock: {:?}",
        kernel.get_attribute(FunctionAttribute::MaxThreadsPerBlock)
    );
    println!(
        "SharedMemorySizeBytes: {:?}",
        kernel.get_attribute(FunctionAttribute::SharedMemorySizeBytes)
    );
    println!(
        "ConstSizeBytes: {:?}",
        kernel.get_attribute(FunctionAttribute::ConstSizeBytes)
    );
    println!(
        "LocalSizeBytes: {:?}",
        kernel.get_attribute(FunctionAttribute::LocalSizeBytes)
    );
    println!(
        "NumRegisters: {:?}",
        kernel.get_attribute(FunctionAttribute::NumRegisters)
    );
    println!(
        "PtxVersion: {:?}",
        kernel.get_attribute(FunctionAttribute::PtxVersion)
    );
    println!(
        "BinaryVersion: {:?}",
        kernel.get_attribute(FunctionAttribute::BinaryVersion)
    );
    println!(
        "CacheModeCa: {:?}",
        kernel.get_attribute(FunctionAttribute::CacheModeCa)
    );

    println!("{:=^80}", "");
}
