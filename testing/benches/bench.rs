#![feature(test)]
#![cfg_attr(feature = "clippy", feature(plugin))]
#![cfg_attr(feature = "clippy", plugin(clippy))]

extern crate rustc_serialize;
#[macro_use]
extern crate serde;
extern crate test;

include!(concat!(env!("OUT_DIR"), "/bench.rs"));
