#![deny(warnings)]

#[macro_use]
mod macros;

// This test target is convoluted with the actual #[test] in a separate file to
// get it so that the stable compiler does not need to parse the code of the
// test. If the test were written with #[cfg(feature = "versioning")] #[test]
// right here, the stable compiler would fail to parse those raw identifiers
// even if the cfg were not enabled.
#[cfg(feature = "versioning")]
mod versioning;
