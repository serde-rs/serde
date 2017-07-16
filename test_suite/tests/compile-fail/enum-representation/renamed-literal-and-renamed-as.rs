// Copyright 2017 Serde Developers
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#[macro_use]
extern crate serde_derive;

#[derive(Serialize)] //~ ERROR: proc-macro derive panicked
#[serde(tag = "type")]
enum E {
    #[serde(rename = false, rename_as = "bool")] //~^^^ HELP: #[serde(rename = false)] is not compatible with #[serde(rename_as = ...)]. Use a string literal as in #[serde(rename = "false")]
    Newtype(u8),
}

fn main() {}
