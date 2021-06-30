//! This crate provides Serde's two derive macros.
//!
//! ```edition2018
//! # use serde_derive::{Serialize, Deserialize};
//! #
//! #[derive(Serialize, Deserialize)]
//! # struct S;
//! #
//! # fn main() {}
//! ```
//!
//! Please refer to [https://serde.rs/derive.html] for how to set this up and
//! troubleshoot.
//!
//! [https://serde.rs/derive.html]: https://serde.rs/derive.html
//!
//! # Examples
//!
//! ```ignore
//! use serde::{Serialize, Deserialize};
//!
//! #[derive(Serialize, Deserialize, Debug)]
//! struct Point {
//!     x: i32,
//!     y: i32,
//! }
//!
//! fn main() {
//!     let point = Point { x: 1, y: 2 };
//!
//!     // Convert the Point to a JSON string.
//!     let serialized = serde_json::to_string(&point).unwrap();
//!
//!     // Prints serialized = {"x":1,"y":2}
//!     println!("serialized = {}", serialized);
//!
//!     // Convert the JSON string back to a Point.
//!     let deserialized: Point = serde_json::from_str(&serialized).unwrap();
//!
//!     // Prints deserialized = Point { x: 1, y: 2 }
//!     println!("deserialized = {:?}", deserialized);
//! }
//! ```
//!
//! # Attributes
//!
//! [Attributes] are used to customize the `Serialize` and `Deserialize`
//! implementations produced by Serde's derive. They require a Rust compiler
//! version 1.15 or newer.
//!
//! [Attributes]: https://doc.rust-lang.org/book/attributes.html
//!
//! There are three categories of attributes:
//!
//! - [Container attributes](#container-attributes) — apply to a struct or enum declaration.
//! - [Variant attributes](#variant-attributes) — apply to a variant of an enum.
//! - [Field attributes](#field-attributes) — apply to one field in a struct or in an enum variant.
//!
//! ```
//! # extern crate serde;
//! use serde::{Serialize, Deserialize};
//!
//! #[derive(Serialize, Deserialize)]
//! #[serde(deny_unknown_fields)]  // <-- this is a container attribute
//! struct S {
//!     #[serde(default)]  // <-- this is a field attribute
//!     f: i32,
//! }
//!
//! #[derive(Serialize, Deserialize)]
//! #[serde(rename = "e")]  // <-- this is also a container attribute
//! enum E {
//!     #[serde(rename = "a")]  // <-- this is a variant attribute
//!     A(String),
//! }
//! ```
//!
//! Note that a single struct, enum, variant, or field may have multiple
//! attributes on it.
//!
//! ## Container Attributes
//!
//! ### `#[serde(rename = "name")]`
//!
//! Serialize and deserialize this struct or enum with the given name instead of
//! its Rust name.
//!
//! Allows specifying independent names for serialization vs deserialization:
//!
//! - `#[serde(rename(serialize = "ser_name"))]`
//! - `#[serde(rename(deserialize = "de_name"))]`
//! - `#[serde(rename(serialize = "ser_name", deserialize = "de_name"))]`
//!
//! ### `#[serde(rename_all = "...")]`
//!
//! Rename all the fields (if this is a struct) or variants (if this is an enum)
//! according to the given case convention. The possible values are
//! `"lowercase"`, `"UPPERCASE"`, `"PascalCase"`, `"camelCase"`, `"snake_case"`,
//! `"SCREAMING_SNAKE_CASE"`, `"kebab-case"`, `"SCREAMING-KEBAB-CASE"`.
//!
//! Allows specifying independent cases for serialization vs deserialization:
//!
//! - `#[serde(rename_all(serialize = "..."))]`
//! - `#[serde(rename_all(deserialize = "..."))]`
//! - `#[serde(rename_all(serialize = "...", deserialize = "..."))]`
//!
//! ### `#[serde(deny_unknown_fields)]`
//!
//! Always error during deserialization when encountering unknown fields. When
//! this attribute is not present, by default unknown fields are ignored for
//! self-describing formats like JSON.
//!
//! *Note:* this attribute is not supported in combination with [`flatten`],
//! neither on the outer struct nor on the flattened field.
//!
//! [`flatten`]: https://serde.rs/field-attrs.html#flatten
//!
//! ### `#[serde(tag = "type")]`
//!
//! Use the **internally tagged** enum representation for this enum, with the
//! given tag. See
//! [enum representations](https://serde.rs/enum-representations.html)
//! for details on this representation.
//!
//! #### Examples
//!
//! ```ignore
//! # extern crate serde;
//! # use serde::{Serialize, Deserialize};
//! #[derive(Serialize, Deserialize)]
//! #[serde(tag = "type")]
//! enum Message {
//!     Request { id: String, method: String, params: Params },
//!     Response { id: String, result: Value },
//! }
//! ```
//!
//! In JSON, it looks like:
//!
//! ```json
//! {"type": "Request", "id": "...", "method": "...", "params": {...}}
//! ```
//!
//! ### `#[serde(tag = "t", content = "c")]`
//!
//! Use the **adjacently tagged** enum representation for this enum, with the
//! given field names for the tag and content. See
//! [enum representations](https://serde.rs/enum-representations.html) for
//! details on this representation.
//!
//! #### Examples
//!
//! ```ignore
//! # extern crate serde;
//! # use serde::{Serialize, Deserialize};
//! #[derive(Serialize, Deserialize)]
//! #[serde(tag = "t", content = "c")]
//! enum Block {
//!     Para(Vec<Inline>),
//!     Str(String),
//! }
//! ```
//!
//! In JSON, it looks like:
//!
//! ```json
//! {"t": "Para", "c": [{...}, {...}]}
//! {"t": "Str", "c": "the string"}
//! ```
//!
//! ### `#[serde(untagged)]`
//!
//! Use the **untagged** enum representation for this enum. See
//! [enum representations](https://serde.rs/enum-representations.html)
//! for details on this representation.
//!
//! #### Examples
//!
//! ```ignore
//! # extern crate serde;
//! # use serde::{Serialize, Deserialize};
//! #[derive(Serialize, Deserialize)]
//! #[serde(untagged)]
//! enum Message {
//!     Request { id: String, method: String, params: Params },
//!     Response { id: String, result: Value },
//! }
//! ```
//!
//! In JSON, it looks like:
//!
//! ```json
//! {"id": "...", "method": "...", "params": {...}}
//! ```
//!
//! Another variant with raw type.
//!
//! ```
//! # extern crate serde;
//! # use serde::{Serialize, Deserialize};
//! #[derive(Serialize, Deserialize)]
//! #[serde(untagged)]
//! enum Data {
//!     Integer(u64),
//!     Pair(String, String),
//! }
//! ```
//!
//! In JSON, it looks like:
//!
//! ```json
//! 1
//! ["a", "b"]
//! ```
//!
//! ### `#[serde(bound = "T: MyTrait")]`
//!
//! Where-clause for the `Serialize` and `Deserialize` impls. This replaces any
//! trait bounds inferred by Serde.
//!
//! Allows specifying independent bounds for serialization vs deserialization:
//!
//! - `#[serde(bound(serialize = "T: MySerTrait"))]`
//! - `#[serde(bound(deserialize = "T: MyDeTrait"))]`
//! - `#[serde(bound(serialize = "T: MySerTrait", deserialize = "T: MyDeTrait"))]`
//!
//! ### `#[serde(default)]`
//!
//! When deserializing, any missing fields should be filled in from the struct's
//! implementation of `Default`. Only allowed on structs.
//!
//! ### `#[serde(default = "path")]`
//!
//! When deserializing, any missing fields should be filled in from the object
//! returned by the given function or method. The function must be callable as
//! `fn() -> T`. For example `default = "my_default"` would invoke `my_default()`
//! and `default = "SomeTrait::some_default"` would invoke
//! `SomeTrait::some_default()`. Only allowed on structs.
//!
//! ### `#[serde(remote = "...")]`
//!
//! This is used for deriving `Serialize` and `Deserialize` for [remote
//! types](https://serde.rs/remote-derive.html).
//!
//! ### `#[serde(transparent)]`
//!
//! Serialize and deserialize a newtype struct or a braced struct with one field
//! exactly the same as if its one field were serialized and deserialized by
//! itself. Analogous to `#[repr(transparent)]`.
//!
//! ### `#[serde(from = "FromType")]`
//!
//! Deserialize this type by deserializing into `FromType`, then converting. This
//! type must implement `From<FromType>`, and `FromType` must implement
//! `Deserialize`.
//!
//! ### `#[serde(try_from = "FromType")]`
//!
//! Deserialize this type by deserializing into `FromType`, then converting
//! fallibly. This type must implement `TryFrom<FromType>` with an error type that
//! implements `Display`, and `FromType` must implement `Deserialize`.
//!
//! ### `#[serde(into = "IntoType")]`
//!
//! Serialize this type by converting it into the specified `IntoType` and
//! serializing that. This type must implement `Clone` and `Into<IntoType>`, and
//! `IntoType` must implement `Serialize`.
//!
//! ### `#[serde(crate = "...")]`
//!
//! Specify a path to the `serde` crate instance to use when referring to Serde
//! APIs from generated code. This is normally only applicable when invoking
//! re-exported Serde derives from a public macro in a different crate.
//!
//! ## Variant Attributes
//!
//! ### `#[serde(rename = "name")]`
//!
//! Serialize and deserialize this variant with the given name instead of its Rust
//! name.
//!
//! Allows specifying independent names for serialization vs deserialization:
//!
//! - `#[serde(rename(serialize = "ser_name"))]`
//! - `#[serde(rename(deserialize = "de_name"))]`
//! - `#[serde(rename(serialize = "ser_name", deserialize = "de_name"))]`
//!
//! ### `#[serde(alias = "name")]`
//!
//! Deserialize this variant from the given name *or* from its Rust name. May be
//! repeated to specify multiple possible names for the same variant.
//!
//! ### `#[serde(rename_all = "...")]`
//!
//! Rename all the fields of this struct variant according to the given case
//! convention. The possible values are `"lowercase"`, `"UPPERCASE"`,
//! `"PascalCase"`, `"camelCase"`, `"snake_case"`, `"SCREAMING_SNAKE_CASE"`,
//! `"kebab-case"`, `"SCREAMING-KEBAB-CASE"`.
//!
//! Allows specifying independent cases for serialization vs deserialization:
//!
//! - `#[serde(rename_all(serialize = "..."))]`
//! - `#[serde(rename_all(deserialize = "..."))]`
//! - `#[serde(rename_all(serialize = "...", deserialize = "..."))]`
//!
//! ### `#[serde(skip)]`
//!
//! Never serialize or deserialize this variant.
//!
//! ### `#[serde(skip_serializing)]`
//!
//! Never serialize this variant. Trying to serialize this variant is treated as
//! an error.
//!
//! ### `#[serde(skip_deserializing)]`
//!
//! Never deserialize this variant.
//!
//! ### `#[serde(serialize_with = "path")]`
//!
//! Serialize this variant using a function that is different from its
//! implementation of `Serialize`. The given function must be callable as
//! `fn<S>(&FIELD0, &FIELD1, ..., S) -> Result<S::Ok, S::Error> where S:
//! Serializer`, although it may also be generic over the `FIELD{n}` types.
//! Variants used with `serialize_with` are not required to be able to derive
//! `Serialize`.
//!
//! `FIELD{n}` exists for every field of the variant. So a unit variant has just
//! the `S` argument, and tuple/struct variants have an argument for every field.
//!
//! ### `#[serde(deserialize_with = "path")]`
//!
//! Deserialize this variant using a function that is different from its
//! implementation of `Deserialize`. The given function must be callable as
//! `fn<'de, D>(D) -> Result<FIELDS, D::Error> where D: Deserializer<'de>`,
//! although it may also be generic over the elements of `FIELDS`. Variants used
//! with `deserialize_with` are not required be able to derive `Deserialize`.
//!
//! `FIELDS` is a tuple of all fields of the variant. A unit variant will have
//! `()` as its `FIELDS` type.
//!
//! ### `#[serde(with = "module")]`
//!
//! Combination of `serialize_with` and `deserialize_with`. Serde will use
//! `$module::serialize` as the `serialize_with` function and
//! `$module::deserialize` as the `deserialize_with` function.
//!
//! ### `#[serde(bound = "T: MyTrait")]`
//!
//! Where-clause for the `Serialize` and/or `Deserialize` impls. This replaces any
//! trait bounds inferred by Serde for the current variant.
//!
//! Allows specifying independent bounds for serialization vs deserialization:
//!
//! - `#[serde(bound(serialize = "T: MySerTrait"))]`
//! - `#[serde(bound(deserialize = "T: MyDeTrait"))]`
//! - `#[serde(bound(serialize = "T: MySerTrait", deserialize = "T: MyDeTrait"))]`
//!
//! ### `#[serde(borrow)]` and `#[serde(borrow = "'a + 'b + ...")]`
//!
//! Borrow data for this field from the deserializer by using zero-copy
//! deserialization. See
//! [this example](https://serde.rs/lifetimes.html#borrowing-data-in-a-derived-impl).
//! Only allowed on a newtype variant (a tuple variant with only one field).
//!
//! ### `#[serde(other)]`
//!
//! Deserialize this variant if the enum tag is anything other than the tag of one
//! of the other variants in this enum. Only allowed on a unit variant inside of
//! an internally tagged or adjacently tagged enum.
//!
//! For example if we have an internally tagged enum with `serde(tag = "variant")`
//! containing variants `A`, `B`, and `Unknown` marked `serde(other)`, the
//! `Unknown` variant would be deserialized any time the `"variant"` field of the
//! input is neither `"A"` nor `"B"`.
//!
//! ## Field Attributes
//!
//! ### `#[serde(rename = "name")]`
//!
//! Serialize and deserialize this field with the given name instead of its Rust
//! name. This is useful for
//! [serializing fields as camelCase](https://serde.rs/attr-rename.html)
//! or serializing fields with names that are reserved Rust keywords.
//!
//! Allows specifying independent names for serialization vs deserialization:
//!
//! - `#[serde(rename(serialize = "ser_name"))]`
//! - `#[serde(rename(deserialize = "de_name"))]`
//! - `#[serde(rename(serialize = "ser_name", deserialize = "de_name"))]`
//!
//! ### `#[serde(alias = "name")]`
//!
//! Deserialize this field from the given name *or* from its Rust name. May be
//! repeated to specify multiple possible names for the same field.
//!
//! ### `#[serde(default)]`
//!
//! If the value is not present when deserializing, use the
//! `Default::default()`.
//!
//! ### `#[serde(default = "path")]`
//!
//! If the value is not present when deserializing, call a function to get a
//! default value. The given function must be callable as `fn() -> T`. For
//! example `default = "empty_value"` would invoke `empty_value()` and
//! `default = "SomeTrait::some_default"` would invoke
//! `SomeTrait::some_default()`.
//!
//! ### `#[serde(flatten)]`
//!
//! Flatten the contents of this field into the container it is defined in.
//!
//! This removes one level of structure between the serialized representation
//! and the Rust data structure representation. It can be used for factoring
//! common keys into a shared structure, or for capturing remaining fields into
//! a map with arbitrary string keys. The
//! [struct flattening](https://serde.rs/attr-flatten.html)
//! page provides some examples.
//!
//! *Note:* this attribute is not supported in combination with structs that use
//! [`deny_unknown_fields`]. Neither the outer nor inner flattened struct should
//! use that attribute.
//!
//! [`deny_unknown_fields`]: https://serde.rs/container-attrs.html#deny_unknown_fields
//!
//! ### `#[serde(skip)]`
//!
//! Skip this field: do not serialize or deserialize it.
//!
//! When deserializing, Serde will use `Default::default()` or the function
//! given by `default = "..."` to get a default value for this field.
//!
//! ### `#[serde(skip_serializing)]`
//!
//! Skip this field when serializing, but not when deserializing.
//!
//! ### `#[serde(skip_deserializing)]`
//!
//! Skip this field when deserializing, but not when serializing.
//!
//! When deserializing, Serde will use `Default::default()` or the function
//! given by `default = "..."` to get a default value for this field.
//!
//! ### `#[serde(skip_serializing_if = "path")]`
//!
//! Call a function to determine whether to skip serializing this field. The
//! given function must be callable as `fn(&T) -> bool`, although it may be
//! generic over `T`. For example `skip_serializing_if = "Option::is_none"`
//! would skip an Option that is None.
//!
//! ### `#[serde(serialize_with = "path")]`
//!
//! Serialize this field using a function that is different from its
//! implementation of `Serialize`. The given function must be callable as
//! `fn<S>(&T, S) -> Result<S::Ok, S::Error> where S: Serializer`, although it
//! may also be generic over `T`. Fields used with `serialize_with` are not
//! required to implement `Serialize`.
//!
//! ### `#[serde(deserialize_with = "path")]`
//!
//! Deserialize this field using a function that is different from its
//! implementation of `Deserialize`. The given function must be callable as
//! `fn<'de, D>(D) -> Result<T, D::Error> where D: Deserializer<'de>`, although
//! it may also be generic over `T`. Fields used with `deserialize_with` are not
//! required to implement `Deserialize`.
//!
//! ### `#[serde(with = "module")]`
//!
//! Combination of `serialize_with` and `deserialize_with`. Serde will use
//! `$module::serialize` as the `serialize_with` function and
//! `$module::deserialize` as the `deserialize_with` function.
//!
//! ### `#[serde(borrow)]` and `#[serde(borrow = "'a + 'b + ...")]`
//!
//! Borrow data for this field from the deserializer by using zero-copy
//! deserialization. See
//! [this example](https://serde.rs/lifetimes.html#borrowing-data-in-a-derived-impl).
//!
//! ### `#[serde(bound = "T: MyTrait")]`
//!
//! Where-clause for the `Serialize` and `Deserialize` impls. This replaces any
//! trait bounds inferred by Serde for the current field.
//!
//! Allows specifying independent bounds for serialization vs deserialization:
//!
//! - `#[serde(bound(serialize = "T: MySerTrait"))]`
//! - `#[serde(bound(deserialize = "T: MyDeTrait"))]`
//! - `#[serde(bound(serialize = "T: MySerTrait", deserialize = "T: MyDeTrait"))]`
//!
//! ### `#[serde(getter = "...")]`
//!
//! This is used when deriving `Serialize` for a
//! [remote type](https://serde.rs/remote-derive.html)
//! that has one or more private fields.

#![doc(html_root_url = "https://docs.rs/serde_derive/1.0.126")]
#![allow(unknown_lints, bare_trait_objects)]
#![deny(clippy::all, clippy::pedantic)]
// Ignored clippy lints
#![allow(
    // clippy false positive: https://github.com/rust-lang/rust-clippy/issues/7054
    clippy::branches_sharing_code,
    clippy::cognitive_complexity,
    clippy::enum_variant_names,
    // clippy bug: https://github.com/rust-lang/rust-clippy/issues/6797
    clippy::manual_map,
    clippy::match_like_matches_macro,
    clippy::needless_pass_by_value,
    clippy::too_many_arguments,
    clippy::trivially_copy_pass_by_ref,
    clippy::used_underscore_binding,
    clippy::wildcard_in_or_patterns,
    // clippy bug: https://github.com/rust-lang/rust-clippy/issues/5704
    clippy::unnested_or_patterns,
)]
// Ignored clippy_pedantic lints
#![allow(
    clippy::cast_possible_truncation,
    clippy::checked_conversions,
    clippy::doc_markdown,
    clippy::enum_glob_use,
    clippy::indexing_slicing,
    clippy::items_after_statements,
    clippy::let_underscore_drop,
    clippy::map_err_ignore,
    clippy::match_same_arms,
    // clippy bug: https://github.com/rust-lang/rust-clippy/issues/6984
    clippy::match_wildcard_for_single_variants,
    clippy::module_name_repetitions,
    clippy::must_use_candidate,
    clippy::option_if_let_else,
    clippy::similar_names,
    clippy::single_match_else,
    clippy::struct_excessive_bools,
    clippy::too_many_lines,
    clippy::unseparated_literal_suffix,
    clippy::unused_self,
    clippy::use_self,
    clippy::wildcard_imports
)]

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
mod dummy;
mod pretend;
mod ser;
mod try;

/// Derive `Serialize`. See [`crate`] for more.
#[proc_macro_derive(Serialize, attributes(serde))]
pub fn derive_serialize(input: TokenStream) -> TokenStream {
    let mut input = parse_macro_input!(input as DeriveInput);
    ser::expand_derive_serialize(&mut input)
        .unwrap_or_else(to_compile_errors)
        .into()
}

/// Derive `Deserialize`. See [`crate`] for more.
#[proc_macro_derive(Deserialize, attributes(serde))]
pub fn derive_deserialize(input: TokenStream) -> TokenStream {
    let mut input = parse_macro_input!(input as DeriveInput);
    de::expand_derive_deserialize(&mut input)
        .unwrap_or_else(to_compile_errors)
        .into()
}

fn to_compile_errors(errors: Vec<syn::Error>) -> proc_macro2::TokenStream {
    let compile_errors = errors.iter().map(syn::Error::to_compile_error);
    quote!(#(#compile_errors)*)
}
