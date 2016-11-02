#![cfg_attr(feature = "clippy", plugin(clippy))]
#![cfg_attr(feature = "clippy", feature(plugin))]
#![cfg_attr(feature = "clippy", allow(too_many_arguments))]
#![cfg_attr(feature = "clippy", allow(used_underscore_binding))]
#![cfg_attr(not(feature = "with-syntex"), feature(rustc_private, plugin))]

// The `quote!` macro requires deep recursion.
#![recursion_limit = "192"]

extern crate serde_codegen_internals as internals;

#[cfg(feature = "with-syntex")]
extern crate syntex;

#[cfg(feature = "with-syntex")]
#[macro_use]
extern crate syntex_syntax as syntax;

#[cfg(not(feature = "with-syntex"))]
#[macro_use]
extern crate syntax;

#[cfg(not(feature = "with-syntex"))]
extern crate rustc_plugin;

extern crate syn;
#[macro_use]
extern crate quote;

#[cfg(feature = "with-syn")]
extern crate post_expansion;

#[cfg(feature = "with-syntex")]
use std::path::Path;

#[cfg(not(feature = "with-syntex"))]
use syntax::feature_gate::AttributeType;

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
                match attr.node.value.node {
                    ast::MetaItemKind::List(ref n, _) if n == &"serde" => { return None; }
                    _ => {}
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

    reg.add_decorator("derive_Serialize", expand_derive_serialize);
    reg.add_decorator("derive_Deserialize", expand_derive_deserialize);

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

#[cfg(not(feature = "with-syntex"))]
pub fn register(reg: &mut rustc_plugin::Registry) {
    reg.register_syntax_extension(
        syntax::parse::token::intern("derive_Serialize"),
        syntax::ext::base::MultiDecorator(
            Box::new(expand_derive_serialize)));

    reg.register_syntax_extension(
        syntax::parse::token::intern("derive_Deserialize"),
        syntax::ext::base::MultiDecorator(
            Box::new(expand_derive_deserialize)));

    reg.register_attribute("serde".to_owned(), AttributeType::Normal);
}

macro_rules! shim {
    ($name:ident $pkg:ident :: $func:ident) => {
        fn $func(
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
                    match attr.node.value.node {
                        ast::MetaItemKind::List(ref name, _) if name == "serde" => {
                            attr::mark_used(attr);
                        }
                        _ => {}
                    }
                }
            }
            visit::walk_item(&mut MarkSerdeAttributesUsed, item);

            use syntax::print::pprust;
            let s = pprust::item_to_string(item);

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

shim!(Serialize ser::expand_derive_serialize);
shim!(Deserialize de::expand_derive_deserialize);

#[cfg(feature = "with-syn")]
pub fn expand_single_item(item: &str) -> Result<String, String> {
    let syn_item = syn::parse_macro_input(item).unwrap();
    let (ser, de, syn_item) = strip_serde_derives(syn_item);
    let expanded_ser = if ser {
        Some(try!(ser::expand_derive_serialize(&syn_item)))
    } else {
        None
    };
    let expanded_de = if de {
        Some(try!(de::expand_derive_deserialize(&syn_item)))
    } else {
        None
    };
    let syn_item = post_expansion::strip_attrs_later(syn_item, &["serde"], "serde");
    return Ok(quote!(#expanded_ser #expanded_de #syn_item).to_string());

    fn strip_serde_derives(item: syn::MacroInput) -> (bool, bool, syn::MacroInput) {
        let mut ser = false;
        let mut de = false;
        let item = syn::MacroInput {
            attrs: item.attrs.into_iter().flat_map(|attr| {
                if attr.is_sugared_doc || attr.style != syn::AttrStyle::Outer {
                    return Some(attr);
                }
                let (name, nested) = match attr.value {
                    syn::MetaItem::List(name, nested) => (name, nested),
                    _ => return Some(attr)
                };
                if name != "derive" {
                    return Some(syn::Attribute {
                        style: syn::AttrStyle::Outer,
                        value: syn::MetaItem::List(name, nested),
                        is_sugared_doc: false,
                    });
                }
                let rest: Vec<_> = nested.into_iter().filter(|nested| {
                    match *nested {
                        syn::MetaItem::Word(ref word) if word == "Serialize" => {
                            ser = true;
                            false
                        }
                        syn::MetaItem::Word(ref word) if word == "Deserialize" => {
                            de = true;
                            false
                        }
                        _ => true,
                    }
                }).collect();
                if rest.is_empty() {
                    None
                } else {
                    Some(syn::Attribute {
                        style: syn::AttrStyle::Outer,
                        value: syn::MetaItem::List(name, rest),
                        is_sugared_doc: false,
                    })
                }
            }).collect(),
            ..item
        };
        (ser, de, item)
    }
}
