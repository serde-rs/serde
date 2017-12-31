// Copyright 2017 Serde Developers
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#![doc(html_root_url = "https://docs.rs/serde_derive_internals/0.19.0")]

extern crate syn;
#[macro_use]
extern crate synom;

pub mod ast;
pub mod attr;

mod ctxt;
pub use ctxt::Ctxt;

mod case;
mod check;
