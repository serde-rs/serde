#![feature(macro_rules, phase)]
#![crate_type = "dylib"]
#![crate_type = "rlib"]

// test harness access
#[cfg(test)]
extern crate test;

#[phase(plugin)]
extern crate serde_macros;

#[cfg(test)]
extern crate serialize;

pub use ser::{Serializer, Serializable};
pub use de::{Deserializer, Deserializable};

pub mod de;
pub mod ser;
pub mod json;

// an inner module so we can use serde_macros.
mod serde {
    pub use de;
    pub use ser;
}
