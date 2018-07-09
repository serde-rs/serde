// Copyright 2017 Serde Developers
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#![doc(html_root_url = "https://docs.rs/serde_derive_internals/0.23.1")]
#![cfg_attr(
    feature = "cargo-clippy",
    allow(
        cyclomatic_complexity,
        doc_markdown,
        match_same_arms,
        redundant_field_names
    )
)]

#[macro_use]
extern crate syn;

extern crate proc_macro2;

#[path = "src/mod.rs"]
mod internals;

pub use internals::*;
