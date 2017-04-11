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

#![doc(html_root_url = "https://docs.serde.rs")]
#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(feature = "unstable", feature(nonzero, specialization, zero_one))]
#![cfg_attr(all(feature = "std", feature = "unstable"), feature(into_boxed_c_str))]
#![cfg_attr(feature = "alloc", feature(alloc))]
#![cfg_attr(feature = "collections", feature(collections))]
#![cfg_attr(feature = "cargo-clippy", allow(linkedlist, type_complexity, doc_markdown))]
#![deny(missing_docs, unused_imports)]

#[cfg(feature = "collections")]
extern crate collections;

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(all(feature = "unstable", feature = "std"))]
extern crate core;

/// A facade around all the types we need from the `std`, `core`, `alloc`, and
/// `collections` crates. This avoids elaborate import wrangling having to
/// happen in every module.
mod lib {
    #[cfg(feature = "std")]
    use std as core;
    #[cfg(not(feature = "std"))]
    use core;

    pub use self::core::{cmp, iter, mem, ops, str};
    pub use self::core::{i8, i16, i32, i64, isize};
    pub use self::core::{u8, u16, u32, u64, usize};
    pub use self::core::{f32, f64};

    pub use self::core::clone::{self, Clone};
    pub use self::core::convert::{self, From, Into};
    pub use self::core::default::{self, Default};
    pub use self::core::fmt::{self, Debug, Display};
    pub use self::core::marker::{self, PhantomData};
    pub use self::core::option::{self, Option};
    pub use self::core::result::{self, Result};

    #[cfg(feature = "std")]
    pub use std::borrow::{Cow, ToOwned};
    #[cfg(all(feature = "collections", not(feature = "std")))]
    pub use collections::borrow::{Cow, ToOwned};

    #[cfg(feature = "std")]
    pub use std::string::String;
    #[cfg(all(feature = "collections", not(feature = "std")))]
    pub use collections::string::{String, ToString};
    
    #[cfg(feature = "std")]
    pub use std::vec::Vec;
    #[cfg(all(feature = "collections", not(feature = "std")))]
    pub use collections::vec::Vec;

    #[cfg(feature = "std")]
    pub use std::boxed::Box;
    #[cfg(all(feature = "alloc", not(feature = "std")))]
    pub use alloc::boxed::Box;

    #[cfg(all(feature = "rc", feature = "std"))]
    pub use std::rc::Rc;
    #[cfg(all(feature = "rc", feature = "alloc", not(feature = "std")))]
    pub use alloc::rc::Rc;

    #[cfg(all(feature = "rc", feature = "std"))]
    pub use std::sync::Arc;
    #[cfg(all(feature = "rc", feature = "alloc", not(feature = "std")))]
    pub use alloc::arc::Arc;

    #[cfg(feature = "std")]
    pub use std::collections::{BinaryHeap, BTreeMap, BTreeSet, LinkedList, VecDeque};
    #[cfg(all(feature = "collections", not(feature = "std")))]
    pub use collections::{BinaryHeap, BTreeMap, BTreeSet, LinkedList, VecDeque};

    #[cfg(feature = "std")]
    pub use std::{error, net, path};

    #[cfg(feature = "std")]
    pub use std::collections::{HashMap, HashSet};
    #[cfg(feature = "std")]
    pub use std::ffi::{CString, CStr, OsString, OsStr};
    #[cfg(feature = "std")]
    pub use std::hash::{Hash, BuildHasher};
    #[cfg(feature = "std")]
    pub use std::io::Write;
    #[cfg(feature = "std")]
    pub use std::time::Duration;

    #[cfg(feature = "unstable")]
    pub use core::nonzero::{NonZero, Zeroable};
    #[cfg(feature = "unstable")]
    #[allow(deprecated)] // required for impl Deserialize for NonZero<T>
    pub use core::num::Zero;
}

#[doc(inline)]
pub use ser::{Serialize, Serializer};
#[doc(inline)]
pub use de::{Deserialize, Deserializer};

#[macro_use]
mod macros;

pub mod de;
pub mod ser;

// Generated code uses these to support no_std. Not public API.
#[doc(hidden)]
pub mod export;

// Helpers used by generated code and doc tests. Not public API.
#[doc(hidden)]
pub mod private;

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
