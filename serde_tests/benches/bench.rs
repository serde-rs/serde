#![feature(test)]
#![cfg_attr(feature = "nightly", feature(plugin))]
#![cfg_attr(feature = "nightly", plugin(clippy))]

extern crate rustc_serialize;
extern crate serde;
extern crate test;

include!(concat!(env!("OUT_DIR"), "/bench.rs"));
