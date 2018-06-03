// Copyright 2017 Serde Developers
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#[macro_use]
extern crate serde_derive;

#[derive(Serialize)]
enum Enum {
    #[serde(serialize_with = "serialize_some_newtype_variant")]
    //~^^^ ERROR: variant `Newtype` cannot have both #[serde(serialize_with)] and a field 0 marked with #[serde(skip_serializing)]
    Newtype(#[serde(skip_serializing)] String),
}

fn main() {}
