use core::marker::PhantomData;

use array2d::Array2D;
use rustacuda::error::CudaError;
use rustacuda::memory::DeviceBuffer;

use necsim_core::cogs::Habitat;

use necsim_impls_no_std::cogs::habitat::in_memory::cuda::InMemoryHabitatCuda;
pub use necsim_impls_no_std::cogs::habitat::in_memory::InMemoryHabitat;

#[allow(clippy::module_name_repetitions)]
pub struct InMemoryHabitatBuilder(PhantomData<()>);

impl InMemoryHabitatBuilder {
    #[debug_ensures(
        old(habitat.num_columns()) == ret.get_extent().width() as usize &&
        old(habitat.num_rows()) == ret.get_extent().height() as usize,
        "habitat extent has the dimension of the habitat array"
    )]
    pub fn from_array2d(habitat: &Array2D<u32>) -> InMemoryHabitat {
        #[allow(clippy::cast_possible_truncation)]
        InMemoryHabitat::new(
            habitat
                .elements_row_major_iter()
                .copied()
                .collect::<Vec<u32>>()
                .into_boxed_slice(),
            habitat.num_columns() as u32,
            habitat.num_rows() as u32,
        )
    }

    /// # Errors
    /// Returns a `CudaError` if an error occurs inside CUDA.
    pub fn lend_to_cuda<O, F: FnOnce(InMemoryHabitatCuda) -> Result<O, CudaError>>(
        habitat: &InMemoryHabitat,
        inner: F,
    ) -> Result<O, CudaError> {
        let mut device_buffer = DeviceBuffer::from_slice(unsafe { habitat.as_ref() })?;

        let extent = habitat.get_extent();

        let habitat_cuda = unsafe {
            InMemoryHabitatCuda::new(
                device_buffer.as_device_ptr(),
                extent.width(),
                extent.height(),
            )
        };

        let result = inner(habitat_cuda);

        match DeviceBuffer::drop(device_buffer) {
            Ok(()) => result,
            Err((err, device_buffer)) => {
                core::mem::forget(device_buffer);
                Err(err)
            }
        }
    }
}
