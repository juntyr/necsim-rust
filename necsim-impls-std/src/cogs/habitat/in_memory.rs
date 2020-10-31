use core::marker::PhantomData;

use array2d::Array2D;

use necsim_core::cogs::Habitat;

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
}
