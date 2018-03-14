// Copyright 2018 Serde Developers
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#[macro_use]
extern crate serde_derive;

#[derive(Serialize)] //~ ERROR: the trait bound `&std::string::String: std::iter::Iterator` is not satisfied
#[serde(repr = "map", unknown_fields_into="other")]
struct X {
    a: u32,
    other: String,
}

fn main() {}
