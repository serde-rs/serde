#![doc(html_root_url = "https://docs.rs/serde_derive_internals/0.25.0")]
#![allow(unknown_lints, bare_trait_objects)]
#![cfg_attr(feature = "cargo-clippy", allow(renamed_and_removed_lints))]
#![cfg_attr(
    feature = "cargo-clippy",
    allow(
        cognitive_complexity,
        redundant_field_names,
        result_unit_err,
        trivially_copy_pass_by_ref,
        wildcard_in_or_patterns,
        // clippy bug: https://github.com/rust-lang/rust-clippy/issues/5704
        unnested_or_patterns,
    )
)]

#[macro_use]
extern crate syn;

extern crate proc_macro2;
extern crate quote;

#[path = "src/mod.rs"]
mod internals;

pub use internals::*;
