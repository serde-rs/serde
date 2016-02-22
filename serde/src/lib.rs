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
#![cfg_attr(feature = "nightly", feature(collections, enumset, nonzero, plugin, step_trait,
                                         zero_one))]
#![cfg_attr(feature = "nightly-testing", plugin(clippy))]
#![cfg_attr(feature = "nightly-testing", allow(linkedlist))]

#![deny(missing_docs)]

#[cfg(feature = "nightly")]
extern crate collections;

#[cfg(feature = "nightly")]
extern crate core;

pub use ser::{Serialize, Serializer};
pub use de::{Deserialize, Deserializer, Error};

pub mod bytes;
pub mod de;
pub mod iter;
pub mod ser;
