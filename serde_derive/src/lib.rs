#![feature(proc_macro, proc_macro_lib)]
#![cfg(not(test))]

extern crate proc_macro;
extern crate serde_codegen;

#[macro_use]
extern crate post_expansion;

use proc_macro::TokenStream;

#[proc_macro_derive(Serialize)]
pub fn derive_serialize(input: TokenStream) -> TokenStream {
    let item = format!("#[derive(Serialize)]\n{}", input);
    match serde_codegen::expand_single_item(&item) {
        Ok(expanded) => expanded.parse().unwrap(),
        Err(msg) => panic!(msg),
    }
}

#[proc_macro_derive(Deserialize)]
pub fn derive_deserialize(input: TokenStream) -> TokenStream {
    let item = format!("#[derive(Deserialize)]\n{}", input);
    match serde_codegen::expand_single_item(&item) {
        Ok(expanded) => expanded.parse().unwrap(),
        Err(msg) => panic!(msg),
    }
}

register_post_expansion!(PostExpansion_serde);
