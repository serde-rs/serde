#![feature(test, rustc_macro, rustc_attrs)]

#[macro_use]
extern crate serde_derive;

extern crate test;

include!("../../testing/tests/test.rs.in");
