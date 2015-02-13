#![feature(collections, core, hash, io, std_misc, plugin, unicode)]
#![plugin(serde_macros)]

extern crate unicode;

pub use de::{Deserializer, Deserialize};
pub use ser::{Serializer, Serialize};

pub mod de;
pub mod ser;
pub mod json;

// an inner module so we can use serde_macros.
mod serde {
    pub use de;
    pub use ser;
}
