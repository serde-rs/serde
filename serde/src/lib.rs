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
//! - [CSV], Comma-separated values is a tabular text file format.
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
//! [CSV]: https://docs.rs/csv

// Serde types in rustdoc of other crates get linked to here.
#![doc(html_root_url = "https://docs.rs/serde/1.0.202")]
// Support using Serde without the standard library!
#![cfg_attr(not(feature = "std"), no_std)]
// Show which crate feature enables conditionally compiled APIs in documentation.
#![cfg_attr(docsrs, feature(doc_cfg))]
#[doc(inline)]
pub use serde_core::*;
// Used by generated code and doc tests. Not public API.
#[doc(hidden)]
#[path = "private/mod.rs"]
pub mod __private;
// Re-export #[derive(Serialize, Deserialize)].
//
// The reason re-exporting is not enabled by default is that disabling it would
// be annoying for crates that provide handwritten impls or data formats. They
// would need to disable default features and then explicitly re-enable std.
#[cfg(feature = "serde_derive")]
extern crate serde_derive;

/// Derive macro available if serde is built with `features = ["derive"]`.
#[cfg(feature = "serde_derive")]
#[cfg_attr(docsrs, doc(cfg(feature = "derive")))]
pub use serde_derive::{Deserialize, Serialize};
#[cfg(feature = "alloc")]
extern crate alloc;
mod lib {
    mod core {
        #[cfg(not(feature = "std"))]
        pub use core::*;
        #[cfg(feature = "std")]
        pub use std::*;
    }

    pub use self::core::{f32, f64};
    pub use self::core::{i16, i32, i64, i8};
    pub use self::core::{ptr, str};
    pub use self::core::{u16, u32, u64, u8, usize};

    #[cfg(any(feature = "std", feature = "alloc"))]
    pub use self::core::slice;

    pub use self::core::clone;
    pub use self::core::convert;
    pub use self::core::default;
    pub use self::core::fmt::{self, Debug, Display, Write as FmtWrite};
    pub use self::core::marker::{self, PhantomData};
    pub use self::core::option;
    pub use self::core::result;

    #[cfg(all(feature = "alloc", not(feature = "std")))]
    pub use alloc::borrow::{Cow, ToOwned};
    #[cfg(feature = "std")]
    pub use std::borrow::Cow;

    #[cfg(all(feature = "alloc", not(feature = "std")))]
    pub use alloc::string::{String, ToString};
    #[cfg(feature = "std")]
    pub use std::string::{String, ToString};

    #[cfg(all(feature = "alloc", not(feature = "std")))]
    pub use alloc::vec::Vec;
    #[cfg(feature = "std")]
    pub use std::vec::Vec;

    #[cfg(all(feature = "alloc", not(feature = "std")))]
    pub use alloc::boxed::Box;
    #[cfg(feature = "std")]
    pub use std::boxed::Box;

    #[cfg(feature = "std")]
    pub use std::error;

    #[cfg(feature = "std")]
    pub use std::io::Write;

    #[cfg(all(feature = "std", no_target_has_atomic, not(no_std_atomic)))]
    pub use std::sync::atomic::{
        AtomicBool, AtomicI16, AtomicI32, AtomicI8, AtomicIsize, AtomicU16, AtomicU32, AtomicU8,
        AtomicUsize, Ordering,
    };
    #[cfg(all(feature = "std", no_target_has_atomic, not(no_std_atomic64)))]
    pub use std::sync::atomic::{AtomicI64, AtomicU64};
}

// None of this crate's error handling needs the `From::from` error conversion
// performed implicitly by the `?` operator or the standard library's `try!`
// macro. This simplified macro gives a 5.5% improvement in compile time
// compared to standard `try!`, and 9% improvement compared to `?`.
#[doc(hidden)]
#[macro_export]
macro_rules! tri {
    ($expr:expr) => {
        match $expr {
            Ok(val) => val,
            Err(err) => return Err(err),
        }
    };
}
