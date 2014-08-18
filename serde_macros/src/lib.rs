#![crate_name = "serde_macros"]
#![crate_type = "dylib"]
#![license = "MIT/ASL2"]

#![feature(plugin_registrar, quote)]

extern crate syntax;
extern crate rustc;

use std::gc::Gc;

use syntax::ast::{
    Ident,
    MetaItem,
    Item,
    Expr,
    MutMutable,
    LitNil,
};
use syntax::ast;
use syntax::codemap::Span;
use syntax::ext::base::{ExtCtxt, ItemDecorator};
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

use rustc::plugin::Registry;

#[plugin_registrar]
#[doc(hidden)]
pub fn plugin_registrar(reg: &mut Registry) {
    reg.register_syntax_extension(
        token::intern("deriving_serializable"),
        ItemDecorator(expand_deriving_serializable));

    reg.register_syntax_extension(
        token::intern("deriving_deserializable"),
        ItemDecorator(expand_deriving_deserializable));
}

fn expand_deriving_serializable(cx: &mut ExtCtxt,
                                sp: Span,
                                mitem: Gc<MetaItem>,
                                item: Gc<Item>,
                                push: |Gc<ast::Item>|) {
    let inline = cx.meta_word(sp, token::InternedString::new("inline"));
    let attrs = vec!(cx.attribute(sp, inline));

    let trait_def = TraitDef {
        span: sp,
        attributes: vec!(),
        path: Path::new(vec!("serde", "ser", "Serializable")),
        additional_bounds: Vec::new(),
        generics: LifetimeBounds::empty(),
        methods: vec!(
            MethodDef {
                name: "serialize",
                generics: LifetimeBounds {
                    lifetimes: Vec::new(),
                    bounds: vec!(
                        (
                            "__S",
                            None,
                            vec!(
                                Path::new_(
                                    vec!("serde", "ser", "Serializer"),
                                    None,
                                    vec!(box Literal(Path::new_local("__E"))),
                                    true
                                )
                            ),
                        ),
                        (
                            "__E",
                            None,
                            vec!(),
                        ),
                    )
                },
                explicit_self: borrowed_explicit_self(),
                args: vec!(Ptr(box Literal(Path::new_local("__S")),
                            Borrowed(None, MutMutable))),
                ret_ty: Literal(
                    Path::new_(
                        vec!("std", "result", "Result"),
                        None,
                        vec!(
                            box Tuple(Vec::new()),
                            box Literal(Path::new_local("__E"))
                        ),
                        true
                    )
                ),
                attributes: attrs,
                combine_substructure: combine_substructure(|a, b, c| {
                    serializable_substructure(a, b, c)
                }),
            })
    };

    trait_def.expand(cx, mitem, item, push)
}

fn serializable_substructure(cx: &ExtCtxt, span: Span,
                          substr: &Substructure) -> Gc<Expr> {
    let serializer = substr.nonself_args[0];

    return match *substr.fields {
        Struct(ref fields) => {
            if fields.is_empty() {
                // unit structs have no fields and need to return `Ok()`
                quote_expr!(cx, Ok(()))
            } else {
                let type_name = cx.expr_str(
                    span,
                    token::get_ident(substr.type_ident)
                );
                let len = fields.len();

                let mut stmts: Vec<Gc<ast::Stmt>> = fields.iter()
                    .enumerate()
                    .map(|(i, &FieldInfo { name, self_, span, .. })| {
                        let name = match name {
                            Some(id) => token::get_ident(id),
                            None => token::intern_and_get_ident(format!("_field{}", i).as_slice()),
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

        EnumMatching(_idx, variant, ref fields) => {
            let type_name = cx.expr_str(
                span,
                token::get_ident(substr.type_ident)
            );
            let variant_name = cx.expr_str(
                span,
                token::get_ident(variant.node.name)
            );
            let len = fields.len();

            let stmts: Vec<Gc<ast::Stmt>> = fields.iter()
                .map(|&FieldInfo { self_, span, .. }| {
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

        _ => cx.bug("expected Struct or EnumMatching in deriving_serializable")
    }
}

pub fn expand_deriving_deserializable(cx: &mut ExtCtxt,
                                      span: Span,
                                      mitem: Gc<MetaItem>,
                                      item: Gc<Item>,
                                      push: |Gc<Item>|) {
    let trait_def = TraitDef {
        span: span,
        attributes: Vec::new(),
        path: Path::new(vec!("serde", "de", "Deserializable")),
        additional_bounds: Vec::new(),
        generics: LifetimeBounds::empty(),
        methods: vec!(
            MethodDef {
                name: "deserialize_token",
                generics: LifetimeBounds {
                    lifetimes: Vec::new(),
                    bounds: vec!(
                        (
                            "__D",
                            None,
                            vec!(
                                Path::new_(
                                    vec!("serde", "de", "Deserializer"),
                                    None,
                                    vec!(
                                        box Literal(Path::new_local("__E"))
                                    ),
                                    true
                                )
                            )
                        ),
                        (
                            "__E",
                            None,
                            vec!(),
                        ),
                    )
                },
                explicit_self: None,
                args: vec!(
                    Ptr(
                        box Literal(Path::new_local("__D")),
                        Borrowed(None, MutMutable)
                    ),
                    Literal(Path::new(vec!("serde", "de", "Token"))),
                ),
                ret_ty: Literal(
                    Path::new_(
                        vec!("std", "result", "Result"),
                        None,
                        vec!(
                            box Self,
                            box Literal(Path::new_local("__E"))
                        ),
                        true
                    )
                ),
                attributes: Vec::new(),
                combine_substructure: combine_substructure(|a, b, c| {
                    deserializable_substructure(a, b, c)
                }),
            })
    };

    trait_def.expand(cx, mitem, item, push)
}

fn deserializable_substructure(cx: &mut ExtCtxt, span: Span,
                               substr: &Substructure) -> Gc<Expr> {
    let deserializer = substr.nonself_args[0];
    let token = substr.nonself_args[1];

    match *substr.fields {
        StaticStruct(_, ref fields) => {
            deserialize_struct(
                cx,
                span,
                substr.type_ident,
                fields,
                deserializer,
                token)
        }
        StaticEnum(_, ref fields) => {
            deserialize_enum(
                cx,
                span,
                substr.type_ident,
                fields.as_slice(),
                deserializer,
                token)
        }
        _ => cx.bug("expected StaticEnum or StaticStruct in deriving(Deserializable)")
    }
}

fn deserialize_struct(
    cx: &ExtCtxt,
    span: Span,
    type_ident: Ident,
    fields: &StaticFields,
    deserializer: Gc<ast::Expr>,
    token: Gc<ast::Expr>
) -> Gc<ast::Expr> {
    let struct_block = deserialize_struct_from_struct(
        cx,
        span,
        type_ident,
        fields,
        deserializer
    );

    let map_block = deserialize_struct_from_map(
        cx,
        span,
        type_ident,
        fields,
        deserializer
    );

    quote_expr!(
        cx,
        match $token {
            ::serde::de::StructStart(_, _) => $struct_block,
            ::serde::de::MapStart(_) => $map_block,
            token => {
                let expected_tokens = [
                    ::serde::de::StructStartKind,
                    ::serde::de::MapStartKind,
                ];
                Err($deserializer.syntax_error(token, expected_tokens))
            }
        }
    )
}

fn deserialize_struct_from_struct(
    cx: &ExtCtxt,
    span: Span,
    type_ident: Ident,
    fields: &StaticFields,
    deserializer: Gc<ast::Expr>
) -> Gc<ast::Expr> {
    let expect_struct_field = cx.ident_of("expect_struct_field");

    let call = deserializable_static_fields(
        cx,
        span,
        type_ident,
        fields,
        |cx, span, name| {
            let name = cx.expr_str(span, name);
            quote_expr!(
                cx,
                try!($deserializer.expect_struct_field($name))
            )
        }
    );

    quote_expr!(cx, {
        let result = $call;
        try!($deserializer.expect_struct_end());
        Ok(result)
    })
}

fn deserialize_struct_from_map(
    cx: &ExtCtxt,
    span: Span,
    type_ident: Ident,
    fields: &StaticFields,
    deserializer: Gc<ast::Expr>
) -> Gc<ast::Expr> {
    let fields = match *fields {
        Unnamed(_) => fail!(),
        Named(ref fields) => fields.as_slice(),
    };

    // Declare each field.
    let let_fields: Vec<Gc<ast::Stmt>> = fields.iter()
        .map(|&(name, span)| {
            quote_stmt!(cx, let mut $name = None)
        })
        .collect();

    // Declare key arms.
    let key_arms: Vec<ast::Arm> = fields.iter()
        .map(|&(name, span)| {
            let s = cx.expr_str(span, token::get_ident(name));
            quote_arm!(cx,
                $s => {
                    $name = Some(
                        try!(::serde::de::Deserializable::deserialize($deserializer))
                    );
                    continue;
                })
        })
        .collect();

    let extract_fields: Vec<Gc<ast::Stmt>> = fields.iter()
        .map(|&(name, span)| {
            let name_str = cx.expr_str(span, token::get_ident(name));
            quote_stmt!(cx,
                let $name = match $name {
                    Some($name) => $name,
                    None => try!($deserializer.missing_field($name_str)),
                };
            )
        })
        .collect();

    let result = cx.expr_struct_ident(
        span,
        type_ident,
        fields.iter()
            .map(|&(name, span)| {
                cx.field_imm(span, name, cx.expr_ident(span, name))
            })
            .collect()
    );

    quote_expr!(cx, {
        $let_fields

        loop {
            let token = match try!($deserializer.expect_token()) {
                ::serde::de::End => { break; }
                token => token,
            };

            {
                let key = match token {
                    ::serde::de::Str(s) => s,
                    ::serde::de::String(ref s) => s.as_slice(),
                    token => {
                        let expected_tokens = [
                            ::serde::de::StrKind,
                            ::serde::de::StringKind,
                        ];
                        return Err($deserializer.syntax_error(token, expected_tokens));
                    }
                };

                match key {
                    $key_arms
                    _ => { }
                }
            }

            try!($deserializer.ignore_field(token))
        }

        $extract_fields
        Ok($result)
    })
}

fn deserialize_enum(
    cx: &ExtCtxt,
    span: Span,
    type_ident: Ident,
    fields: &[(Ident, Span, StaticFields)],
    deserializer: Gc<ast::Expr>,
    token: Gc<ast::Expr>
) -> Gc<ast::Expr> {
    let type_name = cx.expr_str(span, token::get_ident(type_ident));

    let variants = fields.iter()
        .map(|&(name, span, _)| {
            cx.expr_str(span, token::get_ident(name))
        })
        .collect();

    let variants = cx.expr_vec(span, variants);

    let arms: Vec<ast::Arm> = fields.iter()
        .enumerate()
        .map(|(i, &(name, span, ref parts))| {
            let call = deserializable_static_fields(
                cx,
                span,
                name,
                parts,
                |cx, span, _| {
                    quote_expr!(cx, try!($deserializer.expect_enum_elt()))
                }
            );

            quote_arm!(cx, $i => $call,)
        })
        .collect();

    quote_expr!(cx, {
        let i = try!($deserializer.expect_enum_start($token, $type_name, $variants));

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
fn deserializable_static_fields(
    cx: &ExtCtxt,
    span: Span,
    outer_pat_ident: Ident,
    fields: &StaticFields,
    getarg: |&ExtCtxt, Span, token::InternedString| -> Gc<Expr>
) -> Gc<Expr> {
    match *fields {
        Unnamed(ref fields) => {
            if fields.is_empty() {
                cx.expr_ident(span, outer_pat_ident)
            } else {
                let fields = fields.iter().enumerate().map(|(i, &span)| {
                    getarg(
                        cx,
                        span,
                        token::intern_and_get_ident(format!("_field{}", i).as_slice())
                    )
                }).collect();

                cx.expr_call_ident(span, outer_pat_ident, fields)
            }
        }
        Named(ref fields) => {
            // use the field's span to get nicer error messages.
            let fields = fields.iter().map(|&(name, span)| {
                let arg = getarg(
                    cx,
                    span,
                    token::get_ident(name)
                );
                cx.field_imm(span, name, arg)
            }).collect();

            cx.expr_struct_ident(span, outer_pat_ident, fields)
        }
    }
}
