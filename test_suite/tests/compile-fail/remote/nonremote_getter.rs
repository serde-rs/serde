// Copyright 2017 Serde Developers
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#[macro_use]
extern crate serde_derive;

#[derive(Serialize)] //~ ERROR: 12:10: 12:19: #[serde(getter = "...")] can only be used in structs that have #[serde(remote = "...")]
struct S {
    #[serde(getter = "S::get")]
    a: u8,
}

impl S {
    fn get(&self) -> u8 {
        self.a
    }
}

fn main() {}
