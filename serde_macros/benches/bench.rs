#![feature(custom_attribute, custom_derive, plugin, test)]
#![plugin(serde_macros)]

extern crate num;
extern crate rustc_serialize;
extern crate serde;
extern crate test;

include!("../../serde_tests/benches/bench.rs.in");
