#![feature(rustc_macro, rustc_macro_lib)]

extern crate rustc_macro;
extern crate serde_codegen;

use rustc_macro::TokenStream;

#[rustc_macro_derive(Serialize)]
pub fn derive_serialize(input: TokenStream) -> TokenStream {
    let item = format!("#[derive(Serialize)]\n{}", input);
    let expanded = serde_codegen::expand_str(&item).unwrap();
    expanded.parse().unwrap()
}

#[rustc_macro_derive(Deserialize)]
pub fn derive_deserialize(input: TokenStream) -> TokenStream {
    let item = format!("#[derive(Deserialize)]\n{}", input);
    let expanded = serde_codegen::expand_str(&item).unwrap();
    expanded.parse().unwrap()
}
