#![feature(test, custom_attribute, custom_derive, plugin)]
#![plugin(serde_macros)]

extern crate test;

include!("../../testing/tests/test.rs.in");

mod compile_tests;
