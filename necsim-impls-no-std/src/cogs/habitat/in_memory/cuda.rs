use alloc::boxed::Box;

use rustacuda_core::DevicePointer;

use super::InMemoryHabitat;

#[allow(clippy::module_name_repetitions)]
#[derive(DeviceCopy, Debug)]
#[repr(C)]
pub struct InMemoryHabitatCuda {
    habitat: DevicePointer<u32>,
    width: u32,
    height: u32,
}

impl InMemoryHabitatCuda {
    /// # Safety
    /// This is an internal function which should only be used to send an `InMemoryHabitat` to CUDA
    #[must_use]
    pub unsafe fn new(habitat: DevicePointer<u32>, width: u32, height: u32) -> Self {
        Self {
            habitat,
            width,
            height,
        }
    }
}

impl InMemoryHabitat {
    /// # Safety
    /// This is an internal function which should only be used to send an `InMemoryHabitat` to CUDA
    #[must_use]
    pub unsafe fn as_ref(&self) -> &[u32] {
        &self.habitat
    }

    /// # Safety
    /// This function should only be used inside the CUDA kernel to re-establish Rust Safety
    pub unsafe fn with_ref<O, F: FnOnce(&InMemoryHabitat) -> O>(
        habitat: *const InMemoryHabitatCuda,
        inner: F,
    ) -> O {
        // Safe as we will NOT expose mutability
        let habitat_cuda: &mut InMemoryHabitatCuda = &mut *(habitat as *mut InMemoryHabitatCuda);

        let habitat_ptr: *mut u32 = habitat_cuda.habitat.as_raw_mut();
        let habitat_slice: &mut [u32] = core::slice::from_raw_parts_mut(
            habitat_ptr,
            (habitat_cuda.width as usize) * (habitat_cuda.height as usize),
        );

        let habitat = InMemoryHabitat {
            habitat: Box::from_raw(habitat_slice),
            width: habitat_cuda.width,
            height: habitat_cuda.height,
        };

        let result = inner(&habitat);

        // MUST forget about habitat as we do NOT own the box containing the habitat_slice
        core::mem::forget(habitat);

        result
    }
}
