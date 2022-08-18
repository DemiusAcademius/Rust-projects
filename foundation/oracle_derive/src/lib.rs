#[macro_use]
extern crate quote;

#[macro_use]
extern crate syn;

extern crate proc_macro;
use proc_macro::TokenStream;

use syn::{parse_macro_input, DeriveInput};
use quote::quote;

mod internals;

/// Generate Query implementation if form of #[derive(Query)]
/// example:


fn to_compile_errors(errors: Vec<syn::Error>) -> proc_macro2::TokenStream {
    let compile_errors = errors.iter().map(syn::Error::to_compile_error);
    quote!(#(#compile_errors)*)
}