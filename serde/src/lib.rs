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
//! See the Serde website https://serde.rs/ for additional documentation and
//! usage examples.
//!
//! ### Design
//!
//! Where many other languages rely on runtime reflection for serializing data,
//! Serde is instead built on Rust's powerful trait system. A data structure
//! that knows how to serialize and deserialize itself is one that implements
//! Serde's `Serialize` and `Deserialize` traits (or uses Serde's code
//! generation to automatically derive implementations at compile time). This
//! avoids any overhead of reflection or runtime type information. In fact in
//! many situations the interaction between data structure and data format can
//! be completely optimized away by the Rust compiler, leaving Serde
//! serialization to perform roughly the same speed as a handwritten serializer
//! for the specific selection of data structure and data format.
//!
//! ### Data formats
//!
//! The following is a partial list of data formats that have been implemented
//! for Serde by the community.
//!
//! - [JSON](https://github.com/serde-rs/json), the ubiquitous JavaScript Object
//!   Notation used by many HTTP APIs.
//! - [Bincode](https://github.com/TyOverby/bincode), a compact binary format
//!   used for IPC within the Servo rendering engine.
//! - [CBOR](https://github.com/pyfisch/cbor), a Concise Binary Object
//!   Representation designed for small message size without the need for
//!   version negotiation.
//! - [YAML](https://github.com/dtolnay/serde-yaml), a popular human-friendly
//!   configuration language that ain't markup language.
//! - [MessagePack](https://github.com/3Hren/msgpack-rust), an efficient binary
//!   format that resembles a compact JSON.
//! - [TOML](https://github.com/alexcrichton/toml-rs), a minimal configuration
//!   format used by [Cargo](http://doc.crates.io/manifest.html).
//! - [Pickle](https://github.com/birkenfeld/serde-pickle), a format common in
//!   the Python world.
//! - [Hjson](https://github.com/laktak/hjson-rust), a variant of JSON designed
//!   to be readable and writable by humans.
//! - [BSON](https://github.com/zonyitoo/bson-rs), the data storage and network
//!   transfer format used by MongoDB.
//! - [URL](https://github.com/nox/serde_urlencoded), the x-www-form-urlencoded
//!   format.
//! - [XML](https://github.com/serde-rs/xml), the flexible machine-friendly W3C
//!   standard. *(deserialization only)*
//! - [Envy](https://github.com/softprops/envy), a way to deserialize
//!   environment variables into Rust structs. *(deserialization only)*
//! - [Redis](https://github.com/OneSignal/serde-redis), deserialize values from
//!   Redis when using [redis-rs](https://crates.io/crates/redis).
//!   *(deserialization only)*

#![doc(html_root_url="https://docs.serde.rs")]
#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(feature = "unstable", feature(nonzero, specialization, into_boxed_c_str))]
#![cfg_attr(feature = "alloc", feature(alloc))]
#![cfg_attr(feature = "collections", feature(collections))]
#![cfg_attr(feature = "cargo-clippy", allow(linkedlist, type_complexity, doc_markdown))]
#![deny(missing_docs)]

#[cfg(feature = "collections")]
extern crate collections;

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "unstable")]
extern crate core as actual_core;

#[cfg(feature = "std")]
mod core {
    pub use std::{ops, hash, fmt, cmp, marker, mem, i8, i16, i32, i64, u8, u16, u32, u64, isize,
                  usize, f32, f64, char, str, num, slice, iter, cell, default, result, option,
                  clone, convert};
    #[cfg(feature = "unstable")]
    pub use actual_core::nonzero;
}

#[doc(inline)]
pub use ser::{Serialize, Serializer};
#[doc(inline)]
pub use de::{Deserialize, Deserializer};

#[macro_use]
mod macros;

pub mod bytes;
pub mod de;
#[cfg(feature = "std")]
#[doc(hidden)]
pub mod iter;
pub mod ser;
#[cfg_attr(feature = "std", doc(hidden))]
pub mod error;
mod utils;

// Generated code uses these to support no_std. Not public API.
#[doc(hidden)]
pub mod export;

// Re-export #[derive(Serialize, Deserialize)].
//
// This is a workaround for https://github.com/rust-lang/cargo/issues/1286.
// Without this re-export, crates that put Serde derives behind a cfg_attr would
// need to use some silly feature name that depends on both serde and
// serde_derive.
//
//     [features]
//     serde-impls = ["serde", "serde_derive"]
//
//     [dependencies]
//     serde = { version = "1.0", optional = true }
//     serde_derive = { version = "1.0", optional = true }
//
//     # Used like this:
//     # #[cfg(feature = "serde-impls")]
//     # #[macro_use]
//     # extern crate serde_derive;
//     #
//     # #[cfg_attr(feature = "serde-impls", derive(Serialize, Deserialize))]
//     # struct S { /* ... */ }
//
// The re-exported derives allow crates to use "serde" as the name of their
// Serde feature which is more intuitive.
//
//     [dependencies]
//     serde = { version = "1.0", optional = true, features = ["derive"] }
//
//     # Used like this:
//     # #[cfg(feature = "serde")]
//     # #[macro_use]
//     # extern crate serde;
//     #
//     # #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
//     # struct S { /* ... */ }
//
// The reason re-exporting is not enabled by default is that disabling it would
// be annoying for crates that provide handwritten impls or data formats. They
// would need to disable default features and then explicitly re-enable std.
#[cfg(feature = "serde_derive")]
#[allow(unused_imports)]
#[macro_use]
extern crate serde_derive;
#[cfg(feature = "serde_derive")]
#[doc(hidden)]
pub use serde_derive::*;
