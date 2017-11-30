// Copyright 2017 Serde Developers
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! This crate provides Serde's two derive macros.
//!
//! ```rust
//! # #[macro_use]
//! # extern crate serde_derive;
//! #
//! #[derive(Serialize, Deserialize)]
//! # struct S;
//! #
//! # fn main() {}
//! ```
//!
//! Please refer to [https://serde.rs/derive.html] for how to set this up.
//!
//! [https://serde.rs/derive.html]: https://serde.rs/derive.html

#![doc(html_root_url = "https://docs.rs/serde_derive/1.0.23")]

#![cfg_attr(feature = "cargo-clippy", allow(too_many_arguments))]
#![cfg_attr(feature = "cargo-clippy", allow(used_underscore_binding))]

// The `quote!` macro requires deep recursion.
#![recursion_limit = "192"]

extern crate syn;
#[macro_use]
extern crate quote;

extern crate serde_derive_internals as internals;

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
