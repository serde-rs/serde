#![cfg_attr(feature = "clippy", plugin(clippy))]
#![cfg_attr(feature = "clippy", feature(plugin))]
#![cfg_attr(feature = "clippy", allow(too_many_arguments))]
#![cfg_attr(feature = "clippy", allow(used_underscore_binding))]
#![cfg_attr(feature = "with-libsyntax", feature(rustc_private, plugin))]

extern crate serde_codegen_internals as internals;

#[cfg(feature = "with-syntex")]
extern crate syntex;

#[cfg(feature = "with-syntex")]
#[macro_use]
extern crate syntex_syntax as syntax;

#[cfg(feature = "with-libsyntax")]
#[macro_use]
extern crate syntax;

#[cfg(feature = "with-libsyntax")]
extern crate rustc_plugin;

extern crate syn;
#[macro_use]
extern crate quote;

#[cfg(feature = "with-syntex")]
use std::path::Path;

#[cfg(feature = "with-libsyntax")]
use syntax::feature_gate::AttributeType;

mod bound;
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
    reg.add_decorator("derive_Deserialize", de::expand_derive_deserialize);

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

#[cfg(feature = "with-libsyntax")]
pub fn register(reg: &mut rustc_plugin::Registry) {
    reg.register_syntax_extension(
        syntax::parse::token::intern("derive_Serialize"),
        syntax::ext::base::MultiDecorator(
            Box::new(expand_derive_serialize)));

    /*reg.register_syntax_extension(
        syntax::parse::token::intern("derive_Deserialize"),
        syntax::ext::base::MultiDecorator(
            Box::new(de::expand_derive_deserialize)));*/

    reg.register_attribute("serde".to_owned(), AttributeType::Normal);
}

#[cfg(any(feature = "with-syntex", feature = "with-libsyntax"))]
use syntax::ast::MetaItem;
#[cfg(any(feature = "with-syntex", feature = "with-libsyntax"))]
use syntax::codemap::Span;
#[cfg(any(feature = "with-syntex", feature = "with-libsyntax"))]
use syntax::ext::base::{Annotatable, ExtCtxt};

#[cfg(any(feature = "with-syntex", feature = "with-libsyntax"))]
fn expand_derive_serialize(
    cx: &mut ExtCtxt,
    _span: Span,
    meta_item: &MetaItem,
    annotatable: &Annotatable,
    push: &mut FnMut(Annotatable)
) {
    let item = match *annotatable {
        Annotatable::Item(ref item) => item,
        _ => {
            cx.span_err(
                meta_item.span,
                "`#[derive(Serialize)]` may only be applied to structs and enums");
            return;
        }
    };

    use syntax::print::pprust;
    let s = pprust::item_to_string(item);

    let syn_item = syn::parse_item(&s).unwrap();
    let expanded = ser::expand_derive_serialize(&syn_item).to_string();

    use syntax::parse;
    let name = "Serialize".to_string();
    let cfg = Vec::new();
    let sess = parse::ParseSess::new();
    let impl_item = parse::parse_item_from_source_str(name, expanded, cfg, &sess);
    push(Annotatable::Item(impl_item.unwrap().unwrap()));
}

#[cfg(feature = "with-syn")]
pub fn expand_single_item(item: &str) -> String {
    let syn_item = syn::parse_item(item).unwrap();
    let (ser, de, syn_item) = strip_serde_derives(syn_item);
    let expanded_ser = if ser {
        Some(ser::expand_derive_serialize(&syn_item))
    } else {
        None
    };
    let expanded_de = if de {
        unimplemented!()
    } else {
        None::<quote::Tokens>
    };
    let syn_item = strip_serde_attrs(syn_item);
    return quote!(#expanded_ser #expanded_de #syn_item).to_string();

    fn strip_serde_derives(item: syn::Item) -> (bool, bool, syn::Item) {
        let mut ser = false;
        let mut de = false;
        let item = syn::Item {
            attrs: item.attrs.into_iter().flat_map(|attr| {
                if attr.is_sugared_doc {
                    return Some(attr);
                }
                let (name, nested) = match attr.value {
                    syn::MetaItem::List(name, nested) => (name, nested),
                    _ => return Some(attr)
                };
                if name != "derive" {
                    return Some(syn::Attribute {
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
                        value: syn::MetaItem::List(name, rest),
                        is_sugared_doc: false,
                    })
                }
            }).collect(),
            ..item
        };
        (ser, de, item)
    }

    fn strip_serde_attrs(item: syn::Item) -> syn::Item {
        syn::Item {
            attrs: strip_serde_from_attrs(item.attrs),
            body: match item.body {
                syn::Body::Enum(variants) => syn::Body::Enum(
                    variants.into_iter().map(|variant| {
                        syn::Variant {
                            ident: variant.ident,
                            attrs: strip_serde_from_attrs(variant.attrs),
                            data: strip_serde_from_variant_data(variant.data),
                        }
                    }).collect()
                ),
                syn::Body::Struct(variant_data) => syn::Body::Struct(
                    strip_serde_from_variant_data(variant_data)
                ),
            },
            ..item
        }
    }

    fn strip_serde_from_variant_data(data: syn::VariantData) -> syn::VariantData {
        match data {
            syn::VariantData::Struct(fields) => syn::VariantData::Struct(
                fields.into_iter().map(strip_serde_from_field).collect()
            ),
            syn::VariantData::Tuple(fields) => syn::VariantData::Tuple(
                fields.into_iter().map(strip_serde_from_field).collect()
            ),
            syn::VariantData::Unit => syn::VariantData::Unit,
        }
    }

    fn strip_serde_from_field(field: syn::Field) -> syn::Field {
        syn::Field {
            attrs: strip_serde_from_attrs(field.attrs),
            ..field
        }
    }

    fn strip_serde_from_attrs(attrs: Vec<syn::Attribute>) -> Vec<syn::Attribute> {
        attrs.into_iter().filter(|attr| {
            match attr.value {
                syn::MetaItem::List(ref ident, _) => ident != "serde",
                _ => true,
            }
        }).collect()
    }
}
