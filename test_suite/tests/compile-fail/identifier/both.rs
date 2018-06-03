// Copyright 2017 Serde Developers
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#[macro_use]
extern crate serde_derive;

#[derive(Deserialize)]
#[serde(field_identifier, variant_identifier)]
//~^^ ERROR: `field_identifier` and `variant_identifier` cannot both be set
enum F {
    A,
    B,
}

fn main() {}
