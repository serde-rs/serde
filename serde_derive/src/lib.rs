#![cfg_attr(feature = "cargo-clippy", allow(too_many_arguments))]
#![cfg_attr(feature = "cargo-clippy", allow(used_underscore_binding))]

// The `quote!` macro requires deep recursion.
#![recursion_limit = "192"]

extern crate syn;
#[macro_use]
extern crate quote;

extern crate serde_codegen_internals as internals;

extern crate proc_macro;
use proc_macro::TokenStream;

#[macro_use]
mod bound;
#[macro_use]
mod fragment;

mod ser;
mod de;

#[proc_macro_derive(Serialize, attributes(serde))]
pub fn derive_serialize(input: TokenStream) -> TokenStream {
    let input = syn::parse_derive_input(&input.to_string()).unwrap();
    match ser::expand_derive_serialize(&input) {
        Ok(expanded) => expanded.parse().unwrap(),
        Err(msg) => panic!(msg),
    }
}

#[proc_macro_derive(Deserialize, attributes(serde))]
pub fn derive_deserialize(input: TokenStream) -> TokenStream {
    let input = syn::parse_derive_input(&input.to_string()).unwrap();
    match de::expand_derive_deserialize(&input) {
        Ok(expanded) => expanded.parse().unwrap(),
        Err(msg) => panic!(msg),
    }
}
