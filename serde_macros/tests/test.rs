#![feature(test, custom_attribute, custom_derive, plugin)]
#![plugin(serde_macros)]

extern crate num;
extern crate serde;
extern crate test;

include!("../../serde_tests/tests/test.rs.in");

mod compile_tests;
