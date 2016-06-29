#![feature(custom_attribute, custom_derive, plugin, test)]
#![cfg_attr(feature = "clippy", plugin(clippy))]
#![plugin(serde_macros)]

extern crate rustc_serialize;
extern crate serde;
extern crate test;

include!("../../testing/benches/bench.rs.in");
