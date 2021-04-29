#![doc(html_root_url = "https://docs.rs/serde_derive_internals/0.26.0")]
#![allow(unknown_lints, bare_trait_objects)]
#![deny(clippy::all, clippy::pedantic)]
// Ignored clippy lints
#![allow(
    clippy::cognitive_complexity,
    // clippy bug: https://github.com/rust-lang/rust-clippy/issues/6797
    clippy::manual_map,
    clippy::missing_panics_doc,
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
    clippy::let_underscore_drop,
    clippy::match_same_arms,
    // clippy bug: https://github.com/rust-lang/rust-clippy/issues/6984
    clippy::match_wildcard_for_single_variants,
    clippy::missing_errors_doc,
    clippy::module_name_repetitions,
    clippy::must_use_candidate,
    clippy::similar_names,
    clippy::struct_excessive_bools,
    clippy::too_many_lines,
    clippy::unused_self,
    clippy::wildcard_imports
)]

#[macro_use]
extern crate syn;

#[macro_use]
extern crate quote;

extern crate proc_macro2;

#[cfg_attr(serde_build_from_git, path = "../serde_derive/src/")]
#[cfg_attr(not(serde_build_from_git), path = "./src/")]
mod derive {
    pub mod internals;

    #[macro_use]
    pub mod bound;

    #[macro_use]
    pub mod fragment;

    pub mod de;
    pub mod dummy;
    pub mod pretend;
    pub mod ser;
    pub mod try;
}

pub use derive::internals::*;

pub use derive::*;
