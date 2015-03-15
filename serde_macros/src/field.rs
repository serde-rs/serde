use syntax::ast;
use syntax::ext::base::ExtCtxt;
use syntax::ptr::P;

use aster;

fn field_alias(field: &ast::StructField) -> Option<&ast::Lit> {
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
                vals.iter().fold(None, |v, mi| {
                    if let ast::MetaNameValue(ref n, ref lit) = mi.node {
                        if n == &"alias" {
                            Some(lit)
                        } else {
                            v
                        }
                    } else {
                        v
                    }
                })
            } else {
                None
            }
        })
}

pub fn struct_field_strs(
    cx: &ExtCtxt,
    builder: &aster::AstBuilder,
    struct_def: &ast::StructDef,
) -> Vec<P<ast::Expr>> {
    struct_def.fields.iter()
        .map(|field| {
            match field_alias(field) {
                Some(alias) => builder.expr().build_lit(P(alias.clone())),
                None => {
                    match field.node.kind {
                        ast::NamedField(name, _) => {
                            builder.expr().str(name)
                        }
                        ast::UnnamedField(_) => {
                            cx.bug("struct has named and unnamed fields")
                        }
                    }
                }
            }
        })
        .collect()
}

pub fn default_value(field: &ast::StructField) -> bool {
    field.node.attrs.iter()
        .any(|sa| {
             if let ast::MetaItem_::MetaList(ref n, ref vals) = sa.node.value.node {
                 if n == &"serde" {
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
