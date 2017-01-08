#![feature(test)]

#[macro_use]
extern crate serde_derive;

extern crate test;

include!("../../testing/tests/test.rs.in");

mod compile_tests;
