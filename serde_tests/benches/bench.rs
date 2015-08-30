#![feature(test)]

extern crate num;
extern crate rustc_serialize;
extern crate serde;
extern crate test;

include!(concat!(env!("OUT_DIR"), "/bench.rs"));
