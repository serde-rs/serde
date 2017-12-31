// Copyright 2017 Serde Developers
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

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
//! See the Serde website [https://serde.rs/] for additional documentation and
//! usage examples.
//!
//! [https://serde.rs/]: https://serde.rs/
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
//! - [Bincode], a compact binary format
//!   used for IPC within the Servo rendering engine.
//! - [CBOR], a Concise Binary Object Representation designed for small message
//!   size without the need for version negotiation.
//! - [YAML], a popular human-friendly configuration language that ain't markup
//!   language.
//! - [MessagePack], an efficient binary format that resembles a compact JSON.
//! - [TOML], a minimal configuration format used by [Cargo].
//! - [Pickle], a format common in the Python world.
//! - [Hjson], a variant of JSON designed to be readable and writable by humans.
//! - [BSON], the data storage and network transfer format used by MongoDB.
//! - [URL], the x-www-form-urlencoded format.
//! - [XML], the flexible machine-friendly W3C standard.
//!   *(deserialization only)*
//! - [Envy], a way to deserialize environment variables into Rust structs.
//!   *(deserialization only)*
//! - [Redis], deserialize values from Redis when using [redis-rs].
//!   *(deserialization only)*
//!
//! [JSON]: https://github.com/serde-rs/json
//! [Bincode]: https://github.com/TyOverby/bincode
//! [CBOR]: https://github.com/pyfisch/cbor
//! [YAML]: https://github.com/dtolnay/serde-yaml
//! [MessagePack]: https://github.com/3Hren/msgpack-rust
//! [TOML]: https://github.com/alexcrichton/toml-rs
//! [Pickle]: https://github.com/birkenfeld/serde-pickle
//! [Hjson]: https://github.com/laktak/hjson-rust
//! [BSON]: https://github.com/zonyitoo/bson-rs
//! [URL]: https://github.com/nox/serde_urlencoded
//! [XML]: https://github.com/RReverser/serde-xml-rs
//! [Envy]: https://github.com/softprops/envy
//! [Redis]: https://github.com/OneSignal/serde-redis
//! [Cargo]: http://doc.crates.io/manifest.html
//! [redis-rs]: https://crates.io/crates/redis

////////////////////////////////////////////////////////////////////////////////

// Serde types in rustdoc of other crates get linked to here.
#![doc(html_root_url = "https://docs.rs/serde/1.0.27")]
// Support using Serde without the standard library!
#![cfg_attr(not(feature = "std"), no_std)]
// Unstable functionality only if the user asks for it. For tracking and
// discussion of these features please refer to this issue:
//
//    https://github.com/serde-rs/serde/issues/812
#![cfg_attr(feature = "unstable", feature(nonzero, specialization))]
#![cfg_attr(feature = "alloc", feature(alloc))]
#![cfg_attr(feature = "cargo-clippy", deny(clippy, clippy_pedantic))]
// Whitelisted clippy lints
#![cfg_attr(feature = "cargo-clippy",
            allow(cast_lossless, const_static_lifetime, doc_markdown, linkedlist,
                  needless_pass_by_value, type_complexity, unreadable_literal,
                  zero_prefixed_literal))]
// Whitelisted clippy_pedantic lints
#![cfg_attr(feature = "cargo-clippy", allow(
// integer and float ser/de requires these sorts of casts
    cast_possible_truncation,
    cast_possible_wrap,
    cast_precision_loss,
    cast_sign_loss,
// simplifies some macros
    invalid_upcast_comparisons,
// things are often more readable this way
    option_unwrap_used,
    result_unwrap_used,
    shadow_reuse,
    single_match_else,
    stutter,
    use_self,
// not practical
    missing_docs_in_private_items,
// alternative is not stable
    empty_enum,
    use_debug,
))]
// Blacklisted Rust lints.
#![deny(missing_docs, unused_imports)]

////////////////////////////////////////////////////////////////////////////////

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(all(feature = "unstable", feature = "std"))]
extern crate core;

/// A facade around all the types we need from the `std`, `core`, and `alloc`
/// crates. This avoids elaborate import wrangling having to happen in every
/// module.
mod lib {
    mod core {
        #[cfg(feature = "std")]
        pub use std::*;
        #[cfg(not(feature = "std"))]
        pub use core::*;
    }

    pub use self::core::{cmp, iter, mem, ops, slice, str};
    pub use self::core::{isize, i16, i32, i64, i8};
    pub use self::core::{usize, u16, u32, u64, u8};
    pub use self::core::{f32, f64};

    pub use self::core::cell::{Cell, RefCell};
    pub use self::core::clone::{self, Clone};
    pub use self::core::convert::{self, From, Into};
    pub use self::core::default::{self, Default};
    pub use self::core::fmt::{self, Debug, Display};
    pub use self::core::marker::{self, PhantomData};
    pub use self::core::option::{self, Option};
    pub use self::core::result::{self, Result};

    #[cfg(feature = "std")]
    pub use std::borrow::{Cow, ToOwned};
    #[cfg(all(feature = "alloc", not(feature = "std")))]
    pub use alloc::borrow::{Cow, ToOwned};

    #[cfg(feature = "std")]
    pub use std::string::String;
    #[cfg(all(feature = "alloc", not(feature = "std")))]
    pub use alloc::string::{String, ToString};

    #[cfg(feature = "std")]
    pub use std::vec::Vec;
    #[cfg(all(feature = "alloc", not(feature = "std")))]
    pub use alloc::vec::Vec;

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
    pub use std::collections::{BTreeMap, BTreeSet, BinaryHeap, LinkedList, VecDeque};
    #[cfg(all(feature = "alloc", not(feature = "std")))]
    pub use alloc::{BTreeMap, BTreeSet, BinaryHeap, LinkedList, VecDeque};

    #[cfg(feature = "std")]
    pub use std::{error, net};

    #[cfg(feature = "std")]
    pub use std::collections::{HashMap, HashSet};
    #[cfg(feature = "std")]
    pub use std::ffi::{CStr, CString, OsStr, OsString};
    #[cfg(feature = "std")]
    pub use std::hash::{BuildHasher, Hash};
    #[cfg(feature = "std")]
    pub use std::io::Write;
    #[cfg(feature = "std")]
    pub use std::num::Wrapping;
    #[cfg(feature = "std")]
    pub use std::path::{Path, PathBuf};
    #[cfg(feature = "std")]
    pub use std::time::{Duration, SystemTime, UNIX_EPOCH};
    #[cfg(feature = "std")]
    pub use std::sync::{Mutex, RwLock};

    #[cfg(feature = "unstable")]
    pub use core::nonzero::{NonZero, Zeroable};
}

////////////////////////////////////////////////////////////////////////////////

#[macro_use]
mod macros;

pub mod ser;
pub mod de;

#[doc(inline)]
pub use ser::{Serialize, Serializer};
#[doc(inline)]
pub use de::{Deserialize, Deserializer};

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
