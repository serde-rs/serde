#![feature(custom_attribute, custom_derive, plugin, test)]
#![plugin(serde_macros)]

extern crate serde;
extern crate test;

include!("test.rs.in");
