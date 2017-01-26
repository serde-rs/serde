#![cfg_attr(feature = "clippy", plugin(clippy))]
#![cfg_attr(feature = "clippy", feature(plugin))]
#![cfg_attr(feature = "clippy", allow(too_many_arguments))]
#![cfg_attr(feature = "clippy", allow(used_underscore_binding))]

// The `quote!` macro requires deep recursion.
#![recursion_limit = "192"]

extern crate serde_codegen_internals as internals;

extern crate syn;
#[macro_use]
extern crate quote;

mod bound;
mod de;
mod ser;

#[doc(hidden)]
/// Not public API. Use the serde_derive crate.
pub fn expand_derive_serialize(item: &str) -> Result<quote::Tokens, String> {
    let syn_item = syn::parse_macro_input(item).unwrap();
    ser::expand_derive_serialize(&syn_item)
}

#[doc(hidden)]
/// Not public API. Use the serde_derive crate.
pub fn expand_derive_deserialize(item: &str) -> Result<quote::Tokens, String> {
    let syn_item = syn::parse_macro_input(item).unwrap();
    de::expand_derive_deserialize(&syn_item)
}
