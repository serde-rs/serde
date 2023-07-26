#[macro_use]
extern crate quote;
#[macro_use]
extern crate syn;
extern crate proc_macro;
extern crate proc_macro2;

#[macro_use]
mod bound;
#[macro_use]
mod fragment;

mod de;
mod dummy;
mod internals;
mod pretend;
mod ser;
mod this;
mod try;

use proc_macro::TokenStream;
use syn::DeriveInput;

#[proc_macro_derive(Serialize, attributes(serde))]
pub fn derive_serialize(input: TokenStream) -> TokenStream {
    let mut input = parse_macro_input!(input as DeriveInput);
    ser::expand_derive_serialize(&mut input)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

#[proc_macro_derive(Deserialize, attributes(serde))]
pub fn derive_deserialize(input: TokenStream) -> TokenStream {
    let mut input = parse_macro_input!(input as DeriveInput);
    de::expand_derive_deserialize(&mut input)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}
