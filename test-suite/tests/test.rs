#![cfg_attr(feature = "clippy", feature(plugin))]
#![cfg_attr(feature = "clippy", plugin(clippy))]

#![cfg_attr(feature = "unstable-testing", feature(test, non_ascii_idents))]

#[cfg(feature = "unstable-testing")]
extern crate test;

#[macro_use]
extern crate serde_derive;

extern crate serde;
extern crate serde_test;

#[macro_use]
mod macros;

mod test_annotations;
mod test_bytes;
mod test_de;
mod test_gen;
mod test_macros;
mod test_ser;

#[cfg(feature = "unstable-testing")]
mod compile_tests;
