pub mod in_memory;

use necsim_core::cogs::{Habitat, LineageReference, LineageStore};

#[allow(clippy::inline_always, clippy::inline_fn_without_body)]
#[contract_trait]
#[allow(clippy::module_name_repetitions)]
pub trait IncoherentLineageStore<H: Habitat, R: LineageReference<H>>: LineageStore<H, R> {}
