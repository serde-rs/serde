#![feature(convert, core, std_misc, unicode)]

extern crate unicode;

pub use ser::{Serialize, Serializer};
pub use de::{Deserialize, Deserializer, Error};

pub mod ser;
pub mod de;
pub mod json;
pub mod bytes;
