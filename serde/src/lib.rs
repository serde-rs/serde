//! Serde Serialization Framework
//!
//! Serde is a powerful framework that enables serialization libraries to generically serialize
//! Rust data structures without the overhead of runtime type information. In many situations, the
//! handshake protocol between serializers and serializees can be completely optimized away,
//! leaving serde to perform roughly the same speed as a hand written serializer for a specific
//! type.
#![doc(html_root_url="http://erickt.github.io/rust-serde")]

extern crate num;

pub use ser::{Serialize, Serializer};
pub use de::{Deserialize, Deserializer, Error};

pub mod bytes;
pub mod de;
pub mod iter;
pub mod json;
pub mod ser;
