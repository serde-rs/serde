#![feature(test, custom_attribute, custom_derive, plugin)]
#![plugin(serde_macros)]

extern crate serde;
extern crate serde_json;
extern crate test;

include!("../../serde_tests/tests/test.rs.in");
