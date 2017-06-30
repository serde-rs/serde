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
    #[serde(rename = 2.1)] //~^^^ HELP: expected serde rename attribute to be a string, an int or a bool: `rename = "..."` or `rename = 3 or `rename = true`
    Newtype(u8),
}


fn main() {}
