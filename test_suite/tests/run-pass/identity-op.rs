// Copyright 2017 Serde Developers
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#![deny(identity_op)]

#[macro_use]
extern crate serde_derive;

// The derived implementation uses 0+1 to add up the number of fields
// serialized, which Clippy warns about. If the expansion info is registered
// correctly, the Clippy lint is not triggered.
#[derive(Serialize)]
struct A { b: u8 }

fn main() {}
