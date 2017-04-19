// Copyright 2017 Serde Developers
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#![doc(html_root_url = "https://docs.rs/serde_test/0.9.13")]

#[macro_use]
extern crate serde;

mod ser;
mod de;
mod error;

mod token;
mod assert;

pub use token::Token;
pub use assert::{assert_tokens, assert_ser_tokens, assert_ser_tokens_error, assert_de_tokens,
                 assert_de_tokens_error};

// Not public API.
#[doc(hidden)]
pub use de::Deserializer;
