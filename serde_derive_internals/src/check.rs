// Copyright 2017 Serde Developers
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use ast::{Body, Container};
use Ctxt;

/// Cross-cutting checks that require looking at more than a single attrs
/// object. Simpler checks should happen when parsing and building the attrs.
pub fn check(cx: &Ctxt, item: &Container) {
    match item.body {
        Body::Enum(_) => {
            if item.body.has_getter() {
                cx.error("#[serde(getter = \"...\")] is not allowed in an enum");
            }
        }
        Body::Struct(_, _) => {
            if item.body.has_getter() && item.attrs.remote().is_none() {
                cx.error("#[serde(getter = \"...\")] can only be used in structs \
                          that have #[serde(remote = \"...\")]");
            }
        }
    }
}
