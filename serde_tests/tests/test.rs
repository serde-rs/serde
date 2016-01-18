#![cfg_attr(feature = "nightly", feature(plugin))]
#![cfg_attr(feature = "nightly", plugin(clippy))]

extern crate num;
extern crate serde;

include!(concat!(env!("OUT_DIR"), "/test.rs"));
