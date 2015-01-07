#![feature(plugin)]
#![crate_type = "dylib"]
#![crate_type = "rlib"]

// test harness access
#[cfg(test)]
extern crate test;

#[plugin]
extern crate serde_macros;

#[cfg(test)]
extern crate serialize;

extern crate "rustc-serialize" as rustc_serialize;
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
