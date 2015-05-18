#![feature(custom_attribute, custom_derive, plugin, test)]
#![plugin(serde_macros)]

extern crate serde;
extern crate test;

mod syntax {
    include!("bench.rs.in");
}
