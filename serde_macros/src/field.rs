use std::collections::HashMap;

use syntax::ast;
use syntax::attr;
use syntax::ext::base::ExtCtxt;
use syntax::ptr::P;

use aster;

use attr::FieldAttrs;

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

fn field_rename_attrs<'a>(
    builder: &aster::AstBuilder,
    field: &'a ast::StructField,
) -> Rename<'a> {
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
                vals.iter().fold(None, |v, mi| {
                    v.or(rename(builder, mi))
                })
            } else {
                None
            }
        })
        .unwrap_or(Rename::None)
}

pub fn struct_field_attrs(
    cx: &ExtCtxt,
    builder: &aster::AstBuilder,
    struct_def: &ast::StructDef,
) -> Vec<FieldAttrs> {
    struct_def.fields.iter()
        .map(|field| {
            match field_rename_attrs(builder, field) {
                Rename::Global(rename) =>
                    FieldAttrs::new(
                        builder.expr().build_lit(P(rename.clone()))),
                Rename::Format(renames) => {
                    let mut res = HashMap::new();
                    res.extend(
                        renames.into_iter()
                            .map(|(k,v)|
                                 (k, builder.expr().build_lit(P(v.clone())))));
                    FieldAttrs::new_with_formats(
                        default_field(cx, builder, field.node.kind),
                        res)
                },
                Rename::None => {
                    FieldAttrs::new(
                        default_field(cx, builder, field.node.kind))
                }
            }
        })
        .collect()
}

fn default_field(
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


pub fn default_value(field: &ast::StructField) -> bool {
    field.node.attrs.iter()
        .any(|sa| {
             if let ast::MetaItem_::MetaList(ref n, ref vals) = sa.node.value.node {
                 if n == &"serde" {
                     attr::mark_used(&sa);
                     vals.iter()
                         .map(|mi|
                              if let ast::MetaItem_::MetaWord(ref n) = mi.node {
                                  n == &"default"
                              } else {
                                  false
                              })
                         .any(|x| x)
                 } else {
                     false
                 }
             }
             else {
                 false
             }
        })
}
