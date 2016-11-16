#![feature(proc_macro, proc_macro_lib)]
#![cfg(not(test))]

extern crate proc_macro;
extern crate serde_codegen;

use proc_macro::TokenStream;

#[proc_macro_derive(Serialize, attributes(serde))]
pub fn derive_serialize(input: TokenStream) -> TokenStream {
    match serde_codegen::expand_derive_serialize(&input.to_string()) {
        Ok(expanded) => expanded.parse().unwrap(),
        Err(msg) => panic!(msg),
    }
}

#[proc_macro_derive(Deserialize, attributes(serde))]
pub fn derive_deserialize(input: TokenStream) -> TokenStream {
    match serde_codegen::expand_derive_deserialize(&input.to_string()) {
        Ok(expanded) => expanded.parse().unwrap(),
        Err(msg) => panic!(msg),
    }
}
