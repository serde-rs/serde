//! Serde Serialization Framework
//!
//! Serde is a powerful framework that enables serialization libraries to generically serialize
//! Rust data structures without the overhead of runtime type information. In many situations, the
//! handshake protocol between serializers and serializees can be completely optimized away,
//! leaving serde to perform roughly the same speed as a hand written serializer for a specific
//! type.
//!
//! For a detailed tutorial on the different ways to use serde please check out the
//! [github repository](https://github.com/serde-rs/serde)

#![doc(html_root_url="https://docs.serde.rs")]
#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(feature = "unstable", feature(nonzero, inclusive_range, zero_one))]
#![cfg_attr(feature = "alloc", feature(alloc))]
#![cfg_attr(feature = "collections", feature(collections, enumset))]
#![cfg_attr(feature = "clippy", feature(plugin))]
#![cfg_attr(feature = "clippy", plugin(clippy))]
#![cfg_attr(feature = "clippy", allow(linkedlist, type_complexity))]
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
            usize, f32, f64, char, str, num, slice, iter, cell, default, result};
    #[cfg(feature = "unstable")]
    pub use actual_core::nonzero;
}

pub use ser::{Serialize, Serializer};
pub use de::{Deserialize, Deserializer};

#[macro_use]
mod macros;

pub mod bytes;
pub mod de;
#[cfg(feature = "std")]
pub mod iter;
pub mod ser;
#[cfg_attr(feature = "std", doc(hidden))]
pub mod error;
mod utils;

// Generated code uses these to support no_std. Not public API.
#[doc(hidden)]
pub mod export;
