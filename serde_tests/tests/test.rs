#![feature(test)]

extern crate serde;
extern crate test;

include!(concat!(env!("OUT_DIR"), "/test.rs"));
