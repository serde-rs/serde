#![feature(macro_rules, phase)]

extern crate collections;

// test harness access
#[cfg(test)]
extern crate test;
#[phase(syntax, link)]
extern crate log;

#[cfg(test)]
extern crate serialize;

pub mod de;
//pub mod json;

//#[cfg(test)]
//pub mod bench_bytes;

//#[cfg(test)]
//pub mod bench_enum;

#[cfg(test)]
pub mod bench_struct;

//#[cfg(test)]
//pub mod bench_vec;
