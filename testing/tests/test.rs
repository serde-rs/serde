#![feature(specialization)]
#![cfg_attr(feature = "clippy", feature(plugin))]
#![cfg_attr(feature = "clippy", plugin(clippy))]

include!(concat!(env!("OUT_DIR"), "/test.rs"));
