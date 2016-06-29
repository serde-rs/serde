#![cfg_attr(feature = "nightly", feature(plugin))]
#![cfg_attr(feature = "nightly", plugin(clippy))]

include!(concat!(env!("OUT_DIR"), "/test.rs"));
