//! # Serde
//!
//! Serde is a framework for ***ser***ializing and ***de***serializing Rust data
//! structures efficiently and generically.
//!
//! The Serde ecosystem consists of data structures that know how to serialize
//! and deserialize themselves along with data formats that know how to
//! serialize and deserialize other things. Serde provides the layer by which
//! these two groups interact with each other, allowing any supported data
//! structure to be serialized and deserialized using any supported data format.
//!
//! See the Serde website <https://serde.rs/> for additional documentation and
//! usage examples.
//!
//! ## Design
//!
//! Where many other languages rely on runtime reflection for serializing data,
//! Serde is instead built on Rust's powerful trait system. A data structure
//! that knows how to serialize and deserialize itself is one that implements
//! Serde's `Serialize` and `Deserialize` traits (or uses Serde's derive
//! attribute to automatically generate implementations at compile time). This
//! avoids any overhead of reflection or runtime type information. In fact in
//! many situations the interaction between data structure and data format can
//! be completely optimized away by the Rust compiler, leaving Serde
//! serialization to perform the same speed as a handwritten serializer for the
//! specific selection of data structure and data format.
//!
//! ## Data formats
//!
//! The following is a partial list of data formats that have been implemented
//! for Serde by the community.
//!
//! - [JSON], the ubiquitous JavaScript Object Notation used by many HTTP APIs.
//! - [Postcard], a no\_std and embedded-systems friendly compact binary format.
//! - [CBOR], a Concise Binary Object Representation designed for small message
//!   size without the need for version negotiation.
//! - [YAML], a self-proclaimed human-friendly configuration language that ain't
//!   markup language.
//! - [MessagePack], an efficient binary format that resembles a compact JSON.
//! - [TOML], a minimal configuration format used by [Cargo].
//! - [Pickle], a format common in the Python world.
//! - [RON], a Rusty Object Notation.
//! - [BSON], the data storage and network transfer format used by MongoDB.
//! - [Avro], a binary format used within Apache Hadoop, with support for schema
//!   definition.
//! - [JSON5], a superset of JSON including some productions from ES5.
//! - [URL] query strings, in the x-www-form-urlencoded format.
//! - [Starlark], the format used for describing build targets by the Bazel and
//!   Buck build systems. *(serialization only)*
//! - [Envy], a way to deserialize environment variables into Rust structs.
//!   *(deserialization only)*
//! - [Envy Store], a way to deserialize [AWS Parameter Store] parameters into
//!   Rust structs. *(deserialization only)*
//! - [S-expressions], the textual representation of code and data used by the
//!   Lisp language family.
//! - [D-Bus]'s binary wire format.
//! - [FlexBuffers], the schemaless cousin of Google's FlatBuffers zero-copy
//!   serialization format.
//! - [Bencode], a simple binary format used in the BitTorrent protocol.
//! - [Token streams], for processing Rust procedural macro input.
//!   *(deserialization only)*
//! - [DynamoDB Items], the format used by [rusoto_dynamodb] to transfer data to
//!   and from DynamoDB.
//! - [Hjson], a syntax extension to JSON designed around human reading and
//!   editing. *(deserialization only)*
//!
//! [JSON]: https://github.com/serde-rs/json
//! [Postcard]: https://github.com/jamesmunns/postcard
//! [CBOR]: https://github.com/enarx/ciborium
//! [YAML]: https://github.com/dtolnay/serde-yaml
//! [MessagePack]: https://github.com/3Hren/msgpack-rust
//! [TOML]: https://docs.rs/toml
//! [Pickle]: https://github.com/birkenfeld/serde-pickle
//! [RON]: https://github.com/ron-rs/ron
//! [BSON]: https://github.com/mongodb/bson-rust
//! [Avro]: https://docs.rs/apache-avro
//! [JSON5]: https://github.com/callum-oakley/json5-rs
//! [URL]: https://docs.rs/serde_qs
//! [Starlark]: https://github.com/dtolnay/serde-starlark
//! [Envy]: https://github.com/softprops/envy
//! [Envy Store]: https://github.com/softprops/envy-store
//! [Cargo]: https://doc.rust-lang.org/cargo/reference/manifest.html
//! [AWS Parameter Store]: https://docs.aws.amazon.com/systems-manager/latest/userguide/systems-manager-parameter-store.html
//! [S-expressions]: https://github.com/rotty/lexpr-rs
//! [D-Bus]: https://docs.rs/zvariant
//! [FlexBuffers]: https://github.com/google/flatbuffers/tree/master/rust/flexbuffers
//! [Bencode]: https://github.com/P3KI/bendy
//! [Token streams]: https://github.com/oxidecomputer/serde_tokenstream
//! [DynamoDB Items]: https://docs.rs/serde_dynamo
//! [rusoto_dynamodb]: https://docs.rs/rusoto_dynamodb
//! [Hjson]: https://github.com/Canop/deser-hjson

// Support using Serde without the standard library!
#![cfg_attr(not(feature = "std"), no_std)]
#[doc(inline)]
pub use serde_core::*;

// Re-export #[derive(Serialize, Deserialize)].
//
// The reason re-exporting is not enabled by default is that disabling it would
// be annoying for crates that provide handwritten impls or data formats. They
// would need to disable default features and then explicitly re-enable std.
#[cfg(feature = "serde_derive")]
extern crate serde_derive;

/// Derive macro available if serde is built with `features = ["derive"]`.
#[cfg(feature = "serde_derive")]
pub use serde_derive::{Deserialize, Serialize};
