use rust_cuda::rustacuda::{
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

pub fn print_kernel_function_attributes(name: &str, function: &Function) {
    println!("{:=^80}", format!(" {name} Kernel Function Attributes "));

    println!(
        "MaxThreadsPerBlock: {:?}",
        function.get_attribute(FunctionAttribute::MaxThreadsPerBlock)
    );
    println!(
        "SharedMemorySizeBytes: {:?}",
        function.get_attribute(FunctionAttribute::SharedMemorySizeBytes)
    );
    println!(
        "ConstSizeBytes: {:?}",
        function.get_attribute(FunctionAttribute::ConstSizeBytes)
    );
    println!(
        "LocalSizeBytes: {:?}",
        function.get_attribute(FunctionAttribute::LocalSizeBytes)
    );
    println!(
        "NumRegisters: {:?}",
        function.get_attribute(FunctionAttribute::NumRegisters)
    );
    println!(
        "PtxVersion: {:?}",
        function.get_attribute(FunctionAttribute::PtxVersion)
    );
    println!(
        "BinaryVersion: {:?}",
        function.get_attribute(FunctionAttribute::BinaryVersion)
    );
    println!(
        "CacheModeCa: {:?}",
        function.get_attribute(FunctionAttribute::CacheModeCa)
    );

    println!("{:=^80}", "");
}
