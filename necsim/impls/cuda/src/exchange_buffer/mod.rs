mod common;
#[cfg(target_os = "cuda")]
mod device;
#[cfg(not(target_os = "cuda"))]
mod host;

#[cfg(target_os = "cuda")]
pub use device::CudaExchangeBufferDevice as CudaExchangeBuffer;
#[cfg(not(target_os = "cuda"))]
pub use host::CudaExchangeBufferHost as CudaExchangeBuffer;

pub use common::CudaExchangeBufferCudaRepresentation;
