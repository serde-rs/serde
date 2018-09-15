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

#![doc(html_root_url = "https://docs.rs/serde_derive/1.0.79")]
#![cfg_attr(feature = "cargo-clippy", allow(renamed_and_removed_lints))]
#![cfg_attr(feature = "cargo-clippy", deny(clippy, clippy_pedantic))]
// Whitelisted clippy lints
#![cfg_attr(
    feature = "cargo-clippy",
    allow(
        cyclomatic_complexity,
        enum_variant_names,
        needless_pass_by_value,
        redundant_field_names,
        too_many_arguments,
        used_underscore_binding,
    )
)]
// Whitelisted clippy_pedantic lints
#![cfg_attr(
    feature = "cargo-clippy",
    allow(
        cast_possible_truncation,
        doc_markdown,
        enum_glob_use,
        filter_map,
        indexing_slicing,
        items_after_statements,
        match_same_arms,
        similar_names,
        single_match_else,
        stutter,
        unseparated_literal_suffix,
        use_self,
    )
)]
// The `quote!` macro requires deep recursion.
#![recursion_limit = "512"]

#[macro_use]
extern crate quote;
#[macro_use]
extern crate syn;

extern crate proc_macro;
extern crate proc_macro2;

mod internals;

use proc_macro::TokenStream;
use syn::DeriveInput;

#[macro_use]
mod bound;
#[macro_use]
mod fragment;

mod de;
mod pretend;
mod ser;
mod try;

#[proc_macro_derive(Serialize, attributes(serde))]
pub fn derive_serialize(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    ser::expand_derive_serialize(&input)
        .unwrap_or_else(compile_error)
        .into()
}

#[proc_macro_derive(Deserialize, attributes(serde))]
pub fn derive_deserialize(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    de::expand_derive_deserialize(&input)
        .unwrap_or_else(compile_error)
        .into()
}

fn compile_error(message: String) -> proc_macro2::TokenStream {
    quote! {
        compile_error!(#message);
    }
}
