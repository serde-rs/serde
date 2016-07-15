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

#![doc(html_root_url="https://serde-rs.github.io/serde/serde")]
#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(feature = "nightly", feature(reflect_marker, unicode, nonzero, plugin, step_trait, zero_one))]
#![cfg_attr(feature = "alloc", feature(alloc))]
#![cfg_attr(feature = "collections", feature(collections, enumset))]
#![cfg_attr(feature = "nightly-testing", plugin(clippy))]
#![cfg_attr(feature = "nightly-testing", allow(linkedlist))]

#![cfg_attr(any(not(feature = "std"), feature = "nightly"), allow(unused_variables, unused_imports, unused_features, dead_code))]

#![deny(missing_docs)]

#[cfg(all(feature = "nightly", feature = "collections"))]
extern crate collections;

#[cfg(all(feature = "nightly", feature = "alloc"))]
extern crate alloc;

#[cfg(feature = "std")]
mod core {
    pub use std::{char, cmp, f32, f64, fmt, hash, i16, i32, i64, i8, isize, iter, marker, mem,
                  num, ops, slice, str, u16, u32, u64, u8, usize};
    #[cfg(feature = "nightly")]
    extern crate core;
    #[cfg(feature = "nightly")]
    pub use self::core::nonzero;
}

pub use ser::{Serialize, Serializer};
pub use de::{Deserialize, Deserializer, Error};

#[cfg(not(feature = "std"))]
macro_rules! format {
    ($s:expr, $($rest:tt)*) => ($s)
}

pub mod bytes;
pub mod de;
#[cfg(feature = "std")]
pub mod iter;
pub mod ser;
#[cfg(not(feature = "std"))]
pub mod error;
mod utils;
