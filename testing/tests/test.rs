#![cfg_attr(feature = "clippy", feature(plugin))]
#![cfg_attr(feature = "clippy", plugin(clippy))]

#![cfg_attr(feature = "unstable-testing", feature(non_ascii_idents))]

include!(concat!(env!("OUT_DIR"), "/test.rs"));
