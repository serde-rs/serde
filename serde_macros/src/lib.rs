#![feature(plugin_registrar, rustc_private)]
#![cfg_attr(feature = "clippy", feature(plugin))]
#![cfg_attr(feature = "clippy", plugin(clippy))]

extern crate rustc_plugin;
extern crate syntax;
extern crate syntex;
extern crate serde_codegen;

use syntax::ast::{self, MetaItem};
use syntax::attr;
use syntax::codemap::Span;
use syntax::ext::base::{Annotatable, ExtCtxt};
use syntax::feature_gate::AttributeType;
use syntax::parse;
use syntax::print::pprust;

fn expand_derive_serialize(
    cx: &mut ExtCtxt,
    span: Span,
    meta_item: &MetaItem,
    annotatable: &Annotatable,
    push: &mut FnMut(Annotatable)
) {
    expand(
        "Serialize",
        serde_codegen::register_serialize,
        cx,
        span,
        meta_item,
        annotatable,
        push)
}

fn expand_derive_deserialize(
    cx: &mut ExtCtxt,
    span: Span,
    meta_item: &MetaItem,
    annotatable: &Annotatable,
    push: &mut FnMut(Annotatable)
) {
    expand(
        "Deserialize",
        serde_codegen::register_deserialize,
        cx,
        span,
        meta_item,
        annotatable,
        push)
}

fn expand<F>(derive_name: &str,
             register: F,
             cx: &mut ExtCtxt,
             _span: Span,
             meta_item: &MetaItem,
             annotatable: &Annotatable,
             push: &mut FnMut(Annotatable))
    where F: Fn(&mut syntex::Registry)
{
    let item = match *annotatable {
        Annotatable::Item(ref item) => item,
        _ => {
            cx.span_err(
                meta_item.span,
                &format!(
                    "`#[derive({})]` may only be applied to structs and enums",
                    &derive_name));
            return;
        }
    };

    let item_str = pprust::item_to_string(&item);

    let mut registry = syntex::Registry::new();
    register(&mut registry);

    let crate_name = cx.crate_root.clone().unwrap_or("<no source>");
    let filename = cx.filename.clone().unwrap_or_else(|| "<no source>".to_string());

    let syntex_src = match registry.expand_str(crate_name, &filename, &item_str) {
        Ok(src) => src,
        Err(err) => {
            cx.span_err(meta_item.span, &err.to_string());
            return;
        }
    };

    let mut parser = parse::new_parser_from_source_str(
        cx.parse_sess(),
        cx.cfg(),
        filename,
        syntex_src);

    // After we've parsed the item, we need to explicitly mark all the serde attributes as used or
    // else the compiler will report an error.
    mark_serde_attributes_used(&item);

    // FIXME: This is a horrible hack that works around the fact there is no easy way yet to delete
    // the item that we're annotating.  At the moment, we know that serde_codegen guarantees that
    // the first item returned in the string will be will be our annotated item, so just skip it.
    if let None = parser.parse_item().expect("no items returned?") {
        cx.span_bug(
            meta_item.span,
            &format!(
                "`#[derive({})]` didn't generate a struct?",
                &derive_name));
    }

    while let Some(item) = parser.parse_item().expect("no items returned?") {
        push(Annotatable::Item(item));
    }
}

/// Strip the serde attributes from the crate.
fn mark_serde_attributes_used(item: &ast::Item) {
    use syntax::visit;

    /// Helper folder that strips the serde attributes after the extensions have been expanded.
    struct MarkSerdeAttributeVisitor;

    impl<'a> visit::Visitor<'a> for MarkSerdeAttributeVisitor {
        fn visit_attribute(&mut self, attr: &'a ast::Attribute) {
            match attr.node.value.node {
                ast::MetaItemKind::List(ref n, _) if n == &"serde" => {
                    attr::mark_used(&attr);
                }
                _ => {}
            }
        }

        fn visit_mac(&mut self, mac: &'a ast::Mac) {
            visit::walk_mac(self, mac)
        }
    }

    visit::Visitor::visit_item(&mut MarkSerdeAttributeVisitor, item)
}

#[plugin_registrar]
#[doc(hidden)]
pub fn plugin_registrar(reg: &mut rustc_plugin::Registry) {
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
