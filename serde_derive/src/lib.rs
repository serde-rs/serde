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

#![doc(html_root_url = "https://docs.rs/serde_derive/1.0.37")]
#![cfg_attr(feature = "cargo-clippy", allow(enum_variant_names, redundant_field_names,
                                            too_many_arguments, used_underscore_binding))]
// The `quote!` macro requires deep recursion.
#![recursion_limit = "512"]

#[macro_use]
extern crate quote;
#[macro_use]
extern crate syn;

extern crate serde_derive_internals as internals;

extern crate proc_macro;
extern crate proc_macro2;

use proc_macro::TokenStream;
use syn::DeriveInput;

// Quote's default is def_site but it appears call_site is likely to stabilize
// before def_site. Thus we try to use only call_site.
macro_rules! quote {
    ($($tt:tt)*) => {
        quote_spanned!($crate::proc_macro2::Span::call_site()=> $($tt)*)
    }
}

#[macro_use]
mod bound;
#[macro_use]
mod fragment;

mod ser;
mod de;

#[proc_macro_derive(Serialize, attributes(serde))]
pub fn derive_serialize(input: TokenStream) -> TokenStream {
    let input: DeriveInput = syn::parse(input).unwrap();
    match ser::expand_derive_serialize(&input) {
        Ok(expanded) => expanded.into(),
        Err(msg) => panic!(msg),
    }
}

#[proc_macro_derive(Deserialize, attributes(serde))]
pub fn derive_deserialize(input: TokenStream) -> TokenStream {
    let input: DeriveInput = syn::parse(input).unwrap();
    match de::expand_derive_deserialize(&input) {
        Ok(expanded) => expanded.into(),
        Err(msg) => panic!(msg),
    }
}
