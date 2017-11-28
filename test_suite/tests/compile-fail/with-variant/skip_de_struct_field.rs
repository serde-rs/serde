// Copyright 2017 Serde Developers
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#[macro_use]
extern crate serde_derive;

#[derive(Deserialize)] //~ ERROR: proc-macro derive panicked
//~^ HELP: variant `Struct` cannot have both #[serde(deserialize_with)] and a field `f1` marked with #[serde(skip_deserializing)]
enum Enum {
    #[serde(deserialize_with = "deserialize_some_other_variant")]
    Struct {
        #[serde(skip_deserializing)]
        f1: String,
        f2: u8,
    },
}

fn main() { }
