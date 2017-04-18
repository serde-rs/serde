// Copyright 2017 Serde Developers
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use ast::{Body, Container, Style};
use attr::Identifier;
use Ctxt;

/// Cross-cutting checks that require looking at more than a single attrs
/// object. Simpler checks should happen when parsing and building the attrs.
pub fn check(cx: &Ctxt, cont: &Container) {
    check_getter(cx, cont);
    check_identifier(cx, cont);
}

/// Getters are only allowed inside structs (not enums) with the `remote`
/// attribute.
fn check_getter(cx: &Ctxt, cont: &Container) {
    match cont.body {
        Body::Enum(_) => {
            if cont.body.has_getter() {
                cx.error("#[serde(getter = \"...\")] is not allowed in an enum");
            }
        }
        Body::Struct(_, _) => {
            if cont.body.has_getter() && cont.attrs.remote().is_none() {
                cx.error(
                    "#[serde(getter = \"...\")] can only be used in structs \
                          that have #[serde(remote = \"...\")]",
                );
            }
        }
    }
}

/// The `other` attribute must be used at most once and it must be the last
/// variant of an enum that has the `field_identifier` attribute.
///
/// Inside a `variant_identifier` all variants must be unit variants. Inside a
/// `field_identifier` all but possibly one variant must be unit variants. The
/// last variant may be a newtype variant which is an implicit "other" case.
fn check_identifier(cx: &Ctxt, cont: &Container) {
    let variants = match cont.body {
        Body::Enum(ref variants) => variants,
        Body::Struct(_, _) => {
            return;
        }
    };

    for (i, variant) in variants.iter().enumerate() {
        match (variant.style, cont.attrs.identifier(), variant.attrs.other()) {
            // The `other` attribute may only be used in a field_identifier.
            (_, Identifier::Variant, true) |
            (_, Identifier::No, true) => {
                cx.error("#[serde(other)] may only be used inside a field_identifier");
            }

            // Variant with `other` attribute must be the last one.
            (Style::Unit, Identifier::Field, true) => {
                if i < variants.len() - 1 {
                    cx.error("#[serde(other)] must be the last variant");
                }
            }

            // Variant with `other` attribute must be a unit variant.
            (_, Identifier::Field, true) => {
                cx.error("#[serde(other)] must be on a unit variant");
            }

            // Any sort of variant is allowed if this is not an identifier.
            (_, Identifier::No, false) => {}

            // Unit variant without `other` attribute is always fine.
            (Style::Unit, _, false) => {}

            // The last field is allowed to be a newtype catch-all.
            (Style::Newtype, Identifier::Field, false) => {
                if i < variants.len() - 1 {
                    cx.error(format!("`{}` must be the last variant", variant.ident));
                }
            }

            (_, Identifier::Field, false) => {
                cx.error("field_identifier may only contain unit variants");
            }

            (_, Identifier::Variant, false) => {
                cx.error("variant_identifier may only contain unit variants");
            }
        }
    }
}
