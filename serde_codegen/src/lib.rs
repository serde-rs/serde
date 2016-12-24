#![cfg_attr(feature = "clippy", plugin(clippy))]
#![cfg_attr(feature = "clippy", feature(plugin))]
#![cfg_attr(feature = "clippy", allow(too_many_arguments))]
#![cfg_attr(feature = "clippy", allow(used_underscore_binding))]

// The `quote!` macro requires deep recursion.
#![recursion_limit = "192"]

extern crate serde_codegen_internals as internals;

#[cfg(feature = "with-syntex")]
extern crate syntex;

#[cfg(feature = "with-syntex")]
#[macro_use]
extern crate syntex_syntax as syntax;

extern crate syn;
#[macro_use]
extern crate quote;

#[cfg(feature = "with-syntex")]
use std::path::Path;

mod bound;
mod de;
mod ser;

#[cfg(feature = "with-syntex")]
fn syntex_registry() -> syntex::Registry {
    use syntax::{ast, fold};

    /// Strip the serde attributes from the crate.
    #[cfg(feature = "with-syntex")]
    fn strip_attributes(krate: ast::Crate) -> ast::Crate {
        /// Helper folder that strips the serde attributes after the extensions have been expanded.
        struct StripAttributeFolder;

        impl fold::Folder for StripAttributeFolder {
            fn fold_attribute(&mut self, attr: ast::Attribute) -> Option<ast::Attribute> {
                if attr.value.name == "serde" {
                    if let ast::MetaItemKind::List(..) = attr.value.node {
                        return None;
                    }
                }
                Some(attr)
            }

            fn fold_mac(&mut self, mac: ast::Mac) -> ast::Mac {
                fold::noop_fold_mac(mac, self)
            }
        }

        fold::Folder::fold_crate(&mut StripAttributeFolder, krate)
    }

    let mut reg = syntex::Registry::new();

    reg.add_attr("feature(custom_derive)");
    reg.add_attr("feature(custom_attribute)");

    reg.add_decorator("derive_Serialize", shim::expand_derive_serialize);
    reg.add_decorator("derive_Deserialize", shim::expand_derive_deserialize);

    reg.add_post_expansion_pass(strip_attributes);

    reg
}

#[cfg(feature = "with-syntex")]
pub fn expand_str(src: &str) -> Result<String, syntex::Error> {
    let src = src.to_owned();

    let expand_thread = move || {
        syntex_registry().expand_str("", "", &src)
    };

    syntex::with_extra_stack(expand_thread)
}

#[cfg(feature = "with-syntex")]
pub fn expand<S, D>(src: S, dst: D) -> Result<(), syntex::Error>
    where S: AsRef<Path>,
          D: AsRef<Path>,
{
    let src = src.as_ref().to_owned();
    let dst = dst.as_ref().to_owned();

    let expand_thread = move || {
        syntex_registry().expand("", src, dst)
    };

    syntex::with_extra_stack(expand_thread)
}

macro_rules! shim {
    ($name:ident $pkg:ident :: $func:ident) => {
        pub fn $func(
            cx: &mut ::syntax::ext::base::ExtCtxt,
            span: ::syntax::codemap::Span,
            meta_item: &::syntax::ast::MetaItem,
            annotatable: &::syntax::ext::base::Annotatable,
            push: &mut FnMut(::syntax::ext::base::Annotatable)
        ) {
            let item = match *annotatable {
                ::syntax::ext::base::Annotatable::Item(ref item) => item,
                _ => {
                    cx.span_err(
                        meta_item.span,
                        concat!("`#[derive(",
                                stringify!($name),
                                ")]` may only be applied to structs and enums"));
                    return;
                }
            };

            use syntax::{attr, ast, visit};
            struct MarkSerdeAttributesUsed;
            impl visit::Visitor for MarkSerdeAttributesUsed {
                fn visit_attribute(&mut self, attr: &ast::Attribute) {
                    if attr.value.name == "serde" {
                        if let ast::MetaItemKind::List(..) = attr.value.node {
                            attr::mark_used(attr);
                        }
                    }
                }
            }
            visit::walk_item(&mut MarkSerdeAttributesUsed, item);

            use syntax::print::pprust;
            let s = pprust::item_to_string(item);

            use {syn, $pkg};
            let syn_item = syn::parse_macro_input(&s).unwrap();
            let expanded = match $pkg::$func(&syn_item) {
                Ok(expanded) => expanded.to_string(),
                Err(msg) => {
                    cx.span_err(span, &msg);
                    return;
                }
            };

            use syntax::parse;
            let name = stringify!($name).to_string();
            let sess = cx.parse_sess;
            let impl_item = parse::parse_item_from_source_str(name, expanded, sess);
            push(::syntax::ext::base::Annotatable::Item(impl_item.unwrap().unwrap()));
        }
    };
}

#[cfg(feature = "with-syntex")]
mod shim {
    shim!(Serialize ser::expand_derive_serialize);
    shim!(Deserialize de::expand_derive_deserialize);
}

#[cfg(feature = "with-syn")]
#[doc(hidden)]
/// Not public API. Use the serde_derive crate.
pub fn expand_derive_serialize(item: &str) -> Result<quote::Tokens, String> {
    let syn_item = syn::parse_macro_input(item).unwrap();
    ser::expand_derive_serialize(&syn_item)
}

#[cfg(feature = "with-syn")]
#[doc(hidden)]
/// Not public API. Use the serde_derive crate.
pub fn expand_derive_deserialize(item: &str) -> Result<quote::Tokens, String> {
    let syn_item = syn::parse_macro_input(item).unwrap();
    de::expand_derive_deserialize(&syn_item)
}
