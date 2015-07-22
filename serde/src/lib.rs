//! Serde Serialization Framework
//!
//! Serde is a powerful framework that enables serialization libraries to generically serialize
//! Rust data structures without the overhead of runtime type information. In many situations, the
//! handshake protocol between serializers and serializees can be completely optimized away,
//! leaving serde to perform roughly the same speed as a hand written serializer for a specific
//! type.
#![doc(html_root_url="http://erickt.github.io/rust-serde")]
#![cfg_attr(feature = "nightly", feature(collections, core, enumset, nonzero, step_trait, vecmap, zero_one))]

extern crate num;

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
