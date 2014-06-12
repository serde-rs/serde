#![feature(macro_rules, phase)]

// test harness access
#[cfg(test)]
extern crate test;

#[phase(plugin, link)]
extern crate log;

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
