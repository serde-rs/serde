use ast::{Body, Item};
use Ctxt;

/// Cross-cutting checks that require looking at more than a single attrs
/// object. Simpler checks should happen when parsing and building the attrs.
pub fn check(cx: &Ctxt, item: &Item) {
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
