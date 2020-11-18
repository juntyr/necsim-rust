use proc_macro::TokenStream;

use std::env;

#[macro_use]
extern crate proc_macro_error;

const SIMULATION_SPECIALISATION_ENV: &'static str = "NECSIM_CUDA_KERNEL_SPECIALISATION";

#[proc_macro_error]
#[proc_macro]
pub fn specialise(item: TokenStream) -> TokenStream {
    match env::var(SIMULATION_SPECIALISATION_ENV) {
        Ok(specialisation) => match format!("{}::{}", item, specialisation).parse() {
            Ok(parsed_specialisation) => parsed_specialisation,
            Err(error) => abort_call_site!("Failed to parse specialisation: {:?}", error),
        },
        Err(error) => abort_call_site!(
            "Failed to read specialisation from {:?}: {:?}",
            SIMULATION_SPECIALISATION_ENV,
            error
        ),
    }
}
