#![crate_name = "serde_macros"]
#![crate_type = "dylib"]

#![feature(plugin_registrar, quote, unboxed_closures, rustc_private)]

extern crate syntax;
extern crate rustc;

use syntax::ast::{
    Attribute,
    Ident,
    MetaItem,
    MetaNameValue,
    Item,
    ItemEnum,
    ItemStruct,
    Expr,
    MutMutable,
    LitStr,
    StructField,
    Variant,
};
use syntax::ast;
use syntax::attr;
use syntax::codemap::Span;
use syntax::ext::base::{ExtCtxt, Decorator, ItemDecorator};
use syntax::ext::build::AstBuilder;
use syntax::ext::deriving::generic::{
    EnumMatching,
    FieldInfo,
    MethodDef,
    Named,
    StaticFields,
    StaticStruct,
    StaticEnum,
    Struct,
    Substructure,
    TraitDef,
    Unnamed,
    combine_substructure,
};
use syntax::ext::deriving::generic::ty::{
    Borrowed,
    LifetimeBounds,
    Literal,
    Path,
    Ptr,
    Self,
    Tuple,
    borrowed_explicit_self,
};
use syntax::parse::token;
use syntax::ptr::P;

use rustc::plugin::Registry;

#[plugin_registrar]
#[doc(hidden)]
pub fn plugin_registrar(reg: &mut Registry) {
    reg.register_syntax_extension(
        token::intern("derive_serialize"),
        Decorator(Box::new(expand_derive_serialize)));

    reg.register_syntax_extension(
        token::intern("derive_deserialize"),
        Decorator(Box::new(expand_derive_deserialize)));
}

fn expand_derive_serialize(cx: &mut ExtCtxt,
                             sp: Span,
                             mitem: &MetaItem,
                             item: &Item,
                             mut push: Box<FnMut(P<ast::Item>)>) {
    let inline = cx.meta_word(sp, token::InternedString::new("inline"));
    let attrs = vec!(cx.attribute(sp, inline));

    let trait_def = TraitDef {
        span: sp,
        attributes: vec!(),
        path: Path::new_(vec!("serde", "ser", "Serialize"), None,
                         vec!(Box::new(Literal(Path::new_local("__S"))),
                              Box::new(Literal(Path::new_local("__E")))), true),
        additional_bounds: Vec::new(),
        generics: LifetimeBounds {
            lifetimes: Vec::new(),
            bounds: vec!(("__S", vec!(Path::new_(
                            vec!("serde", "ser", "Serializer"), None,
                            vec!(Box::new(Literal(Path::new_local("__E")))), true))),
                         ("__E", vec!()))
        },
        methods: vec!(
            MethodDef {
                name: "serialize",
                generics: LifetimeBounds::empty(),
                explicit_self: borrowed_explicit_self(),
                args: vec!(Ptr(Box::new(Literal(Path::new_local("__S"))),
                            Borrowed(None, MutMutable))),
                ret_ty: Literal(
                    Path::new_(
                        vec!("std", "result", "Result"),
                        None,
                        vec!(
                            Box::new(Tuple(Vec::new())),
                            Box::new(Literal(Path::new_local("__E")))
                        ),
                        true
                    )
                ),
                attributes: attrs,
                combine_substructure: combine_substructure(Box::new( |a, b, c| {
                    serialize_substructure(a, b, c, item)
                })),
            }),
        associated_types: vec!()
    };

    trait_def.expand(cx, mitem, item, |item| push.call_mut((item,)))
}

fn serialize_substructure(cx: &ExtCtxt,
                          span: Span,
                          substr: &Substructure,
                          item: &Item) -> P<Expr> {
    let serializer = substr.nonself_args[0].clone();

    match (&item.node, substr.fields) {
        (&ItemStruct(ref definition, _), &Struct(ref fields)) => {
            if fields.is_empty() {
                // unit structs have no fields and need to return `Ok()`
                quote_expr!(cx, Ok(()))
            } else {
                let type_name = cx.expr_str(
                    span,
                    token::get_ident(substr.type_ident)
                );
                let len = fields.len();

                let stmts: Vec<P<ast::Stmt>> = definition.fields.iter()
                    .zip(fields.iter())
                    .enumerate()
                    .map(|(i, (def, &FieldInfo { name, ref self_, span, .. }))| {
                        let serial_name = find_serial_name(def.node.attrs.iter());
                        let name = match (serial_name, name) {
                            (Some(serial), _) => serial.clone(),
                            (None, Some(id)) => token::get_ident(id),
                            (None, None) => token::intern_and_get_ident(format!("_field{}", i).as_slice()),
                        };

                        let name = cx.expr_str(span, name);

                        quote_stmt!(
                            cx,
                            try!($serializer.serialize_struct_elt($name, &$self_))
                        )
                    })
                    .collect();

                quote_expr!(cx, {
                    try!($serializer.serialize_struct_start($type_name, $len));
                    $stmts
                    $serializer.serialize_struct_end()
                })
            }
        }

        (&ItemEnum(ref definition, _), &EnumMatching(_idx, variant, ref fields)) => {
            let type_name = cx.expr_str(
                span,
                token::get_ident(substr.type_ident)
            );
            let variant_name = cx.expr_str(
                span,
                token::get_ident(variant.node.name)
            );
            let len = fields.len();

            let stmts: Vec<P<ast::Stmt>> = definition.variants.iter()
                .zip(fields.iter())
                .map(|(def, &FieldInfo { ref self_, .. })| {
                    let _serial_name = find_serial_name(def.node.attrs.iter());
                    quote_stmt!(
                        cx,
                        try!($serializer.serialize_enum_elt(&$self_))
                    )
                })
                .collect();

            quote_expr!(cx, {
                try!($serializer.serialize_enum_start($type_name, $variant_name, $len));
                $stmts
                $serializer.serialize_enum_end()
            })
        }

        _ => cx.bug("expected Struct or EnumMatching in derive_serialize")
    }
}

pub fn expand_derive_deserialize(cx: &mut ExtCtxt,
                                   span: Span,
                                   mitem: &MetaItem,
                                   item: &Item,
                                   mut push: Box<FnMut(P<Item>)>) {
    let trait_def = TraitDef {
        span: span,
        attributes: Vec::new(),
        path: Path::new_(vec!("serde", "de", "Deserialize"), None,
                         vec!(Box::new(Literal(Path::new_local("__D"))),
                              Box::new(Literal(Path::new_local("__E")))), true),
        additional_bounds: Vec::new(),
        generics: LifetimeBounds {
            lifetimes: Vec::new(),
            bounds: vec!(("__D", vec!(Path::new_(
                            vec!("serde", "de", "Deserializer"), None,
                            vec!(Box::new(Literal(Path::new_local("__E")))), true))),
                         ("__E", vec!()))
        },
        methods: vec!(
            MethodDef {
                name: "deserialize_token",
                generics: LifetimeBounds::empty(),
                explicit_self: None,
                args: vec!(
                    Ptr(
                        Box::new(Literal(Path::new_local("__D"))),
                        Borrowed(None, MutMutable)
                    ),
                    Literal(Path::new(vec!("serde", "de", "Token"))),
                ),
                ret_ty: Literal(
                    Path::new_(
                        vec!("std", "result", "Result"),
                        None,
                        vec!(
                            Box::new(Self),
                            Box::new(Literal(Path::new_local("__E")))
                        ),
                        true
                    )
                ),
                attributes: Vec::new(),
                combine_substructure: combine_substructure(Box::new(|a, b, c| {
                    deserialize_substructure(a, b, c)
                })),
            }),
        associated_types: vec!()
    };

    trait_def.expand(cx, mitem, item, |item| push.call_mut((item,)))
}

fn deserialize_substructure(cx: &mut ExtCtxt,
                            span: Span,
                            substr: &Substructure) -> P<Expr> {
    let deserializer = substr.nonself_args[0].clone();
    let token = substr.nonself_args[1].clone();

    match *substr.fields {
        StaticStruct(ref definition, ref fields) => {
            deserialize_struct(
                cx,
                span,
                substr.type_ident,
                definition.fields.as_slice(),
                fields,
                deserializer.clone(),
                token)
        }
        StaticEnum(ref definition, ref fields) => {
            deserialize_enum(
                cx,
                span,
                substr.type_ident,
                definition.variants.as_slice(),
                fields.as_slice(),
                deserializer,
                token)
        }
        _ => cx.bug("expected StaticEnum or StaticStruct in derive(Deserialize)")
    }
}

fn deserialize_struct(
    cx: &ExtCtxt,
    span: Span,
    type_ident: Ident,
    definitions: &[StructField],
    fields: &StaticFields,
    deserializer: P<ast::Expr>,
    token: P<ast::Expr>
) -> P<ast::Expr> {
    let type_name_str = cx.expr_str(span, token::get_ident(type_ident));

    let fields = match *fields {
        Unnamed(_) => panic!(),
        Named(ref fields) => fields.as_slice(),
    };

    // Convert each field into a unique ident.
    let field_idents: Vec<ast::Ident> = fields.iter()
        .enumerate()
        .map(|(idx, _)| {
            cx.ident_of(format!("field{}", idx).as_slice())
        })
        .collect();

    // Convert each field into their string.
    let field_strs: Vec<P<ast::Expr>> = fields.iter()
        .zip(definitions.iter())
        .map(|(&(name, _), def)| {
            match find_serial_name(def.node.attrs.iter()) {
                Some(serial) => cx.expr_str(span, serial),
                None => cx.expr_str(span, token::get_ident(name)),
            }
        })
        .collect();

    // Declare the static vec slice of field names.
    let static_fields = cx.expr_vec_slice(span, field_strs.clone());

    // Declare each field.
    let let_fields: Vec<P<ast::Stmt>> = field_idents.iter()
        .map(|ident| quote_stmt!(cx, let mut $ident = None))
        .collect();

    // Declare key arms.
    let idx_arms: Vec<ast::Arm> = field_idents.iter()
        .enumerate()
        .map(|(idx, ident)| {
            quote_arm!(cx,
                Some($idx) => { $ident = Some(try!($deserializer.expect_struct_value())); }
            )
        })
        .collect();

    let extract_fields: Vec<P<ast::Stmt>> = field_idents.iter()
        .zip(field_strs.iter())
        .map(|(ident, field_str)| {
            quote_stmt!(cx,
                let $ident = match $ident {
                    Some($ident) => $ident,
                    None => try!($deserializer.missing_field($field_str)),
                };
            )
        })
        .collect();

    let result = cx.expr_struct_ident(
        span,
        type_ident,
        fields.iter()
            .zip(field_idents.iter())
            .map(|(&(name, _), ident)| {
                cx.field_imm(span, name, cx.expr_ident(span, *ident))
            })
            .collect()
    );

    quote_expr!(cx, {
        try!($deserializer.expect_struct_start($token, $type_name_str));

        static FIELDS: &'static [&'static str] = $static_fields;
        $let_fields

        loop {
            let idx = match try!($deserializer.expect_struct_field_or_end(FIELDS)) {
                Some(idx) => idx,
                None => { break; }
            };

            match idx {
                $idx_arms
                Some(_) => unreachable!(),
                None => {
                    let _: ::serde::de::IgnoreTokens =
                        try!(::serde::de::Deserialize::deserialize($deserializer));
                }
            }
            //try!($deserializer.ignore_field(token))
        }

        $extract_fields
        Ok($result)
    })
}

fn deserialize_enum(
    cx: &ExtCtxt,
    span: Span,
    type_ident: Ident,
    definitions: &[P<Variant>],
    fields: &[(Ident, Span, StaticFields)],
    deserializer: P<ast::Expr>,
    token: P<ast::Expr>
) -> P<ast::Expr> {
    let type_name = cx.expr_str(span, token::get_ident(type_ident));

    let serial_names = definitions.iter().map(|def|
        find_serial_name(def.node.attrs.iter())
    ).collect::<Vec<Option<token::InternedString>>>();

    let variants = fields.iter()
        .map(|&(name, span, _)| {
            cx.expr_str(span, token::get_ident(name))
        })
        .collect();

    let variants = cx.expr_vec(span, variants);

    let arms: Vec<ast::Arm> = fields.iter()
        .enumerate()
        .map(|(i, &(name, span, ref parts))| {
            let path = cx.path(span, vec![type_ident, name]);
            let call = deserialize_static_fields(
                cx,
                span,
                path,
                serial_names.as_slice(),
                parts,
                |&: cx, _, _| {
                    quote_expr!(cx, try!($deserializer.expect_enum_elt()))
                }
            );

            quote_arm!(cx, $i => $call,)
        })
        .collect();

    quote_expr!(cx, {
        let i = try!($deserializer.expect_enum_start($token, $type_name, &$variants));

        let result = match i {
            $arms
            _ => { unreachable!() }
        };

        try!($deserializer.expect_enum_end());

        Ok(result)
    })
}

/// Create a deserializer for a single enum variant/struct:
/// - `outer_pat_ident` is the name of this enum variant/struct
/// - `getarg` should retrieve the `uint`-th field with name `&str`.
fn deserialize_static_fields<F>(
    cx: &ExtCtxt,
    span: Span,
    outer_pat_path: ast::Path,
    serial_names: &[Option<token::InternedString>],
    fields: &StaticFields,
    getarg: F
) -> P<Expr> where F: Fn(&ExtCtxt, Span, token::InternedString) -> P<Expr> {
    match *fields {
        Unnamed(ref fields) => {
            let path_expr = cx.expr_path(outer_pat_path);
            if fields.is_empty() {
                path_expr
            } else {
                let fields = fields.iter().enumerate().map(|(i, &span)| {
                    getarg(
                        cx,
                        span,
                        token::intern_and_get_ident(format!("_field{}", i).as_slice())
                    )
                }).collect();

                cx.expr_call(span, path_expr, fields)
            }
        }
        Named(ref fields) => {
            // use the field's span to get nicer error messages.
            let fields = serial_names.iter()
                .zip(fields.iter()).map(|(serial_name, &(name, span))| {
                let effective_name = serial_name.clone().unwrap_or(
                    token::get_ident(name)
                );
                let arg = getarg(
                    cx,
                    span,
                    effective_name
                );
                cx.field_imm(span, name, arg)
            }).collect();

            cx.expr_struct(span, outer_pat_path, fields)
        }
    }
}

fn find_serial_name<'a, I>(mut iterator: I) -> Option<token::InternedString> where
    I: Iterator<Item=&'a Attribute>
{
    for at in iterator {
        match at.node.value.node {
            MetaNameValue(ref at_name, ref value) => {
                match (at_name.get(), &value.node) {
                    ("serial_name", &LitStr(ref string, _)) => {
                        attr::mark_used(at);
                        return Some(string.clone());
                    },
                    _ => ()
                }
            },
            _ => ()
        }
    }
    None
}
