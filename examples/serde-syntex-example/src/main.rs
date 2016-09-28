#![cfg_attr(feature = "serde_derive", feature(rustc_macro))]

#[cfg(feature = "serde_derive")]
#[macro_use]
extern crate serde_derive;

extern crate serde;
extern crate serde_json;

#[cfg(feature = "serde_derive")]
include!("main.rs.in");

#[cfg(not(feature = "serde_derive"))]
include!(concat!(env!("OUT_DIR"), "/main.rs"));
