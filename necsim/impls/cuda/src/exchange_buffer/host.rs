use core::ops::{Deref, DerefMut};

use rustacuda::{
    error::CudaResult,
    memory::{DeviceBox, DeviceBuffer, LockedBuffer},
};

use rustacuda_core::DeviceCopy;

use rust_cuda::{
    common::{DeviceBoxConst, DeviceBoxMut, DeviceOwnedSlice, RustToCuda},
    host::{CudaDropWrapper, EmptyCudaAlloc},
};

use super::CudaExchangeBufferCudaRepresentation;

#[allow(clippy::module_name_repetitions)]
pub struct CudaExchangeBufferHost<T: Clone + DeviceCopy> {
    host_buffer: CudaDropWrapper<LockedBuffer<T>>,
    device_buffer: CudaDropWrapper<DeviceBuffer<T>>,
}

impl<T: Clone + DeviceCopy> Deref for CudaExchangeBufferHost<T> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        self.host_buffer.as_slice()
    }
}

impl<T: Clone + DeviceCopy> DerefMut for CudaExchangeBufferHost<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.host_buffer.as_mut_slice()
    }
}

unsafe impl<T: Clone + DeviceCopy> RustToCuda for CudaExchangeBufferHost<T> {
    type CudaAllocation = rust_cuda::host::NullCudaAlloc;
    type CudaRepresentation = CudaExchangeBufferCudaRepresentation<T>;

    #[allow(clippy::type_complexity)]
    unsafe fn borrow_mut<A: rust_cuda::host::CudaAlloc>(
        &mut self,
        alloc: A,
    ) -> rustacuda::error::CudaResult<(
        Self::CudaRepresentation,
        rust_cuda::host::CombinedCudaAlloc<Self::CudaAllocation, A>,
    )> {
        use rustacuda::memory::CopyDestination;

        self.device_buffer.copy_from(self.host_buffer.as_slice())?;

        Ok((
            CudaExchangeBufferCudaRepresentation(DeviceOwnedSlice::from(&mut self.device_buffer)),
            rust_cuda::host::CombinedCudaAlloc::new(rust_cuda::host::NullCudaAlloc, alloc),
        ))
    }

    #[allow(clippy::type_complexity)]
    unsafe fn un_borrow_mut<A: rust_cuda::host::CudaAlloc>(
        &mut self,
        _cuda_repr: Self::CudaRepresentation,
        alloc: rust_cuda::host::CombinedCudaAlloc<Self::CudaAllocation, A>,
    ) -> rustacuda::error::CudaResult<A> {
        use rustacuda::memory::CopyDestination;

        let (_alloc_front, alloc_tail) = alloc.split();

        self.device_buffer
            .copy_to(self.host_buffer.as_mut_slice())?;

        Ok(alloc_tail)
    }
}

pub struct ExchangeWithCuda<T: RustToCuda<CudaAllocation: EmptyCudaAlloc>> {
    value: T,
    device_box: CudaDropWrapper<DeviceBox<<T as RustToCuda>::CudaRepresentation>>,
}

#[allow(clippy::module_name_repetitions)]
pub struct ExchangeWithHost<T: RustToCuda<CudaAllocation: EmptyCudaAlloc>> {
    value: T,
    device_box: CudaDropWrapper<DeviceBox<<T as RustToCuda>::CudaRepresentation>>,
    cuda_repr: <T as RustToCuda>::CudaRepresentation,
    null_alloc: rust_cuda::host::CombinedCudaAlloc<
        <T as RustToCuda>::CudaAllocation,
        rust_cuda::host::NullCudaAlloc,
    >,
}

impl<T: RustToCuda<CudaAllocation: EmptyCudaAlloc>> ExchangeWithCuda<T> {
    pub fn new(mut value: T) -> CudaResult<Self> {
        let (cuda_repr, _null_alloc) = unsafe { value.borrow_mut(rust_cuda::host::NullCudaAlloc) }?;

        let device_box = CudaDropWrapper::from(DeviceBox::new(&cuda_repr)?);

        Ok(Self { value, device_box })
    }

    pub fn move_to_cuda(mut self) -> CudaResult<ExchangeWithHost<T>> {
        use rustacuda::memory::CopyDestination;

        let (cuda_repr, null_alloc) =
            unsafe { self.value.borrow_mut(rust_cuda::host::NullCudaAlloc) }?;

        self.device_box.copy_from(&cuda_repr)?;

        Ok(ExchangeWithHost {
            value: self.value,
            device_box: self.device_box,
            cuda_repr,
            null_alloc,
        })
    }
}

impl<T: RustToCuda<CudaAllocation: EmptyCudaAlloc>> Deref for ExchangeWithCuda<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<T: RustToCuda<CudaAllocation: EmptyCudaAlloc>> DerefMut for ExchangeWithCuda<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}

impl<T: RustToCuda<CudaAllocation: EmptyCudaAlloc>> ExchangeWithHost<T> {
    pub fn move_to_host(mut self) -> CudaResult<ExchangeWithCuda<T>> {
        let _null_alloc: rust_cuda::host::NullCudaAlloc =
            unsafe { self.value.un_borrow_mut(self.cuda_repr, self.null_alloc) }?;

        Ok(ExchangeWithCuda {
            value: self.value,
            device_box: self.device_box,
        })
    }

    pub fn as_ref(&self) -> DeviceBoxConst<<T as RustToCuda>::CudaRepresentation> {
        DeviceBoxConst::from(&self.device_box)
    }

    pub fn as_mut(&mut self) -> DeviceBoxMut<<T as RustToCuda>::CudaRepresentation> {
        DeviceBoxMut::from(&mut self.device_box)
    }
}
