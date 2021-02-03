#![deny(clippy::pedantic)]

use std::env;

use proc_macro::TokenStream;
use quote::quote;

#[macro_use]
extern crate proc_macro_error;

const SIMULATION_SPECIALISATION_ENV: &str = "NECSIM_CUDA_KERNEL_SPECIALISATION";

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

#[proc_macro]
pub fn rerun_if_specialisation_changed(_item: TokenStream) -> TokenStream {
    let rerun_string = format!(
        "cargo:rerun-if-env-changed={}",
        SIMULATION_SPECIALISATION_ENV
    );

    (quote! {
        println!(#rerun_string);
    })
    .into()
}
