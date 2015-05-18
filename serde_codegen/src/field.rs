use std::collections::HashMap;

use syntax::ast;
use syntax::attr;
use syntax::ext::base::ExtCtxt;
use syntax::ptr::P;

use aster;

use super::attr::FieldAttrs;

enum Rename<'a> {
    None,
    Global(&'a ast::Lit),
    Format(HashMap<P<ast::Expr>, &'a ast::Lit>)
}

fn rename<'a>(
    builder: &aster::AstBuilder,
    mi: &'a ast::MetaItem,
    ) -> Option<Rename<'a>>
{
    match mi.node {
        ast::MetaNameValue(ref n, ref lit) => {
            if n == &"rename" {
                Some(Rename::Global(lit))
            } else {
                None
            }
        },
        ast::MetaList(ref n, ref items) => {
            if n == &"rename" {
                let mut m = HashMap::new();
                m.extend(
                    items.iter()
                        .filter_map(
                            |item|
                            match item.node {
                                ast::MetaNameValue(ref n, ref lit) =>
                                    Some((builder.expr().str(n),
                                          lit)),
                                _ => None
                            }));
                Some(Rename::Format(m))
            } else {
                None
            }
        },
        _ => None
    }
}

pub fn default_value(mi: &ast::MetaItem) -> bool {
    if let ast::MetaItem_::MetaWord(ref n) = mi.node {
        n == &"default"
    } else {
        false
    }
}

fn field_attrs<'a>(
    builder: &aster::AstBuilder,
    field: &'a ast::StructField,
) -> (Rename<'a>, bool) {
    field.node.attrs.iter()
        .find(|sa| {
            if let ast::MetaList(ref n, _) = sa.node.value.node {
                n == &"serde"
            } else {
                false
            }
        })
        .and_then(|sa| {
            if let ast::MetaList(_, ref vals) = sa.node.value.node {
                attr::mark_used(&sa);
                Some((vals.iter()
                      .fold(None, |v, mi| v.or(rename(builder, mi)))
                      .unwrap_or(Rename::None),
                      vals.iter().any(|mi| default_value(mi))))
            } else {
                Some((Rename::None, false))
            }
        })
        .unwrap_or((Rename::None, false))
}

pub fn struct_field_attrs(
    cx: &ExtCtxt,
    builder: &aster::AstBuilder,
    struct_def: &ast::StructDef,
) -> Vec<FieldAttrs> {
    struct_def.fields.iter()
        .map(|field| {
            match field_attrs(builder, field) {
                (Rename::Global(rename), default_value) =>
                    FieldAttrs::new(
                        default_value,
                        builder.expr().build_lit(P(rename.clone()))),
                (Rename::Format(renames), default_value) => {
                    let mut res = HashMap::new();
                    res.extend(
                        renames.into_iter()
                            .map(|(k,v)|
                                 (k, builder.expr().build_lit(P(v.clone())))));
                    FieldAttrs::new_with_formats(
                        default_value,
                        default_field_name(cx, builder, field.node.kind),
                        res)
                },
                (Rename::None, default_value) => {
                    FieldAttrs::new(
                        default_value,
                        default_field_name(cx, builder, field.node.kind))
                }
            }
        })
        .collect()
}

fn default_field_name(
    cx: &ExtCtxt,
    builder: &aster::AstBuilder,
    kind: ast::StructFieldKind,
) -> P<ast::Expr> {
    match kind {
        ast::NamedField(name, _) => {
            builder.expr().str(name)
        }
        ast::UnnamedField(_) => {
            cx.bug("struct has named and unnamed fields")
        }
    }
}
