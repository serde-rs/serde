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
#[serde(untagged)]
enum E {
    #[serde(rename = 3)] //~^^^ HELP: #[serde(rename = int|bool)] cannot be used with #[serde(untagged)]
    Tuple(u8, u8),
}

fn main() {}
