#![feature(macro_rules, phase)]
#![crate_type = "dylib"]
#![crate_type = "rlib"]

// test harness access
#[cfg(test)]
extern crate test;

#[phase(plugin, link)]
extern crate log;

#[phase(plugin)]
extern crate serde_macros;

#[cfg(test)]
extern crate debug;

#[cfg(test)]
extern crate serialize;

pub mod de;
pub mod ser;
pub mod json;

//#[cfg(test)]
//pub mod bench_bytes;

#[cfg(test)]
pub mod bench_enum;

#[cfg(test)]
pub mod bench_struct;

#[cfg(test)]
pub mod bench_vec;

#[cfg(test)]
pub mod bench_map;

#[cfg(test)]
pub mod bench_log;

// an inner module so we can use serde_macros.
mod serde {
    pub use de;
    pub use ser;
}
