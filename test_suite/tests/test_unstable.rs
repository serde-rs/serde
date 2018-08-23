// Copyright 2018 Serde Developers
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#![deny(warnings)]
#![cfg_attr(feature = "unstable", feature(raw_identifiers))]

#[cfg(feature = "unstable")]
#[macro_use]
extern crate serde_derive;

#[cfg(feature = "unstable")]
extern crate serde;
#[cfg(feature = "unstable")]
extern crate serde_test;

// This test target is convoluted with the actual #[test] in a separate file to
// get it so that the stable compiler does not need to parse the code of the
// test. If the test were written with #[cfg(feature = "unstable")] #[test]
// right here, the stable compiler would fail to parse those raw identifiers
// even if the cfg were not enabled.
#[cfg(feature = "unstable")]
mod unstable;
