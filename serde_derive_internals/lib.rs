#![doc(html_root_url = "https://docs.rs/serde_derive_internals/0.25.0")]
#![allow(unknown_lints, bare_trait_objects)]
#![deny(clippy::all, clippy::pedantic)]
// Ignored clippy lints
#![allow(
    clippy::cognitive_complexity,
    clippy::redundant_field_names,
    clippy::result_unit_err,
    clippy::should_implement_trait,
    clippy::trivially_copy_pass_by_ref,
    clippy::wildcard_in_or_patterns,
    // clippy bug: https://github.com/rust-lang/rust-clippy/issues/5704
    clippy::unnested_or_patterns,
)]
// Ignored clippy_pedantic lints
#![allow(
    clippy::doc_markdown,
    clippy::enum_glob_use,
    clippy::items_after_statements,
    clippy::match_same_arms,
    clippy::missing_errors_doc,
    clippy::must_use_candidate,
    clippy::struct_excessive_bools,
    clippy::too_many_lines,
    clippy::wildcard_imports
)]

#[macro_use]
extern crate syn;

extern crate proc_macro2;
extern crate quote;

#[path = "src/mod.rs"]
mod internals;

pub use internals::*;
