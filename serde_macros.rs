#![crate_type = "dylib"]
#![crate_type = "rlib"]

#![feature(plugin_registrar)]

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
                const_nonmatching: true,
                combine_substructure: combine_substructure(|a, b, c| {
                    serializable_substructure(a, b, c)
                }),
            })
    };

    trait_def.expand(cx, mitem, item, push)
}

fn serializable_substructure(cx: &mut ExtCtxt, trait_span: Span,
                          substr: &Substructure) -> Gc<Expr> {
    let serializer = substr.nonself_args[0];

    return match *substr.fields {
        Struct(ref fields) => {
            if fields.is_empty() {
                // unit structs have no fields and need to return `Ok()`
                cx.expr_ok(trait_span, cx.expr_lit(trait_span, LitNil))
            } else {
                let mut stmts = vec!();

                let call = cx.expr_method_call(
                    trait_span,
                    serializer,
                    cx.ident_of("serialize_struct_start"),
                    vec!(
                        cx.expr_str(trait_span, token::get_ident(substr.type_ident)),
                        cx.expr_uint(trait_span, fields.len()),
                    )
                );
                let call = cx.expr_try(trait_span, call);
                stmts.push(cx.stmt_expr(call));

                let emit_struct_sep = cx.ident_of("serialize_struct_sep");

                for (i, &FieldInfo {
                        name,
                        self_,
                        span,
                        ..
                }) in fields.iter().enumerate() {
                    let name = match name {
                        Some(id) => token::get_ident(id),
                        None => token::intern_and_get_ident(format!("_field{}", i).as_slice()),
                    };
                    let call = cx.expr_method_call(
                        span,
                        serializer,
                        emit_struct_sep,
                        vec!(
                            cx.expr_str(span, name),
                            cx.expr_addr_of(span, self_),
                        )
                    );

                    let call = cx.expr_try(span, call);
                    stmts.push(cx.stmt_expr(call));
                }

                let call = cx.expr_method_call(
                    trait_span,
                    serializer,
                    cx.ident_of("serialize_struct_end"),
                    vec!()
                );

                cx.expr_block(cx.block(trait_span, stmts, Some(call)))
            }
        }

        EnumMatching(_idx, variant, ref fields) => {
            let mut stmts = vec!();

            let call = cx.expr_method_call(
                trait_span,
                serializer,
                cx.ident_of("serialize_enum_start"),
                vec!(
                    cx.expr_str(trait_span, token::get_ident(substr.type_ident)),
                    cx.expr_str(trait_span, token::get_ident(variant.node.name)),
                    cx.expr_uint(trait_span, fields.len()),
                )
            );

            let call = cx.expr_try(trait_span, call);
            stmts.push(cx.stmt_expr(call));

            let serialize_struct_sep = cx.ident_of("serialize_enum_sep");

            for &FieldInfo { self_, span, .. } in fields.iter() {
                let call = cx.expr_method_call(
                    span,
                    serializer,
                    serialize_struct_sep,
                    vec!(
                        cx.expr_addr_of(span, self_),
                    )
                );

                let call = cx.expr_try(span, call);

                stmts.push(cx.stmt_expr(call));
            }

            let call = cx.expr_method_call(
                trait_span,
                serializer,
                cx.ident_of("serialize_enum_end"),
                vec!()
            );

            cx.expr_block(cx.block(trait_span, stmts, Some(call)))
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
                const_nonmatching: true,
                combine_substructure: combine_substructure(|a, b, c| {
                    deserializable_substructure(a, b, c)
                }),
            })
    };

    trait_def.expand(cx, mitem, item, push)
}

fn deserializable_substructure(cx: &mut ExtCtxt, trait_span: Span,
                               substr: &Substructure) -> Gc<Expr> {
    let deserializer = substr.nonself_args[0];

    match *substr.fields {
        StaticStruct(_, ref summary) => {
            let mut stmts = vec!();

            let call = cx.expr_method_call(
                trait_span,
                deserializer,
                cx.ident_of("expect_struct_start"),
                vec!(
                    substr.nonself_args[1],
                    cx.expr_str(trait_span, token::get_ident(substr.type_ident)),
                )
            );
            let call = cx.expr_try(trait_span, call);
            stmts.push(cx.stmt_expr(call));

            let expect_struct_field = cx.ident_of("expect_struct_field");

            let call = deserializable_static_fields(
                cx,
                trait_span,
                substr.type_ident,
                summary,
                |cx, span, name| {
                    cx.expr_try(span,
                        cx.expr_method_call(
                            span,
                            deserializer,
                            expect_struct_field,
                            vec!(
                                cx.expr_str(span, name),
                            )
                        )
                    )
                }
            );

            let result = cx.ident_of("result");

            stmts.push(
                cx.stmt_let(
                    trait_span,
                    false,
                    result,
                    call
                )
            );

            let call = cx.expr_method_call(
                trait_span,
                deserializer,
                cx.ident_of("expect_struct_end"),
                vec!()
            );
            let call = cx.expr_try(trait_span, call);
            stmts.push(cx.stmt_expr(call));

            cx.expr_block(
                cx.block(
                    trait_span,
                    stmts,
                    Some(
                        cx.expr_ok(
                            trait_span,
                            cx.expr_ident(trait_span, result)
                        )
                    )
                )
            )
        }
        StaticEnum(_, ref fields) => {
            let mut stmts = vec!();

            let mut arms = vec!();
            let mut variants = vec!();

            let expect_enum_sep = cx.ident_of("expect_enum_sep");
            for (i, &(name, v_span, ref parts)) in fields.iter().enumerate() {
                variants.push(cx.expr_str(v_span, token::get_ident(name)));

                let deserializabled = deserializable_static_fields(cx,
                                                   v_span,
                                                   name,
                                                   parts,
                                                   |cx, span, _| {
                    cx.expr_try(span,
                        cx.expr_method_call(
                            span,
                            deserializer,
                            expect_enum_sep,
                            vec!()
                        )
                    )
                });

                arms.push(
                    cx.arm(
                        v_span,
                        vec!(
                            cx.pat_lit(v_span, cx.expr_uint(v_span, i)),
                        ),
                        deserializabled
                    )
                );
            }

            arms.push(cx.arm_unreachable(trait_span));


            let call = cx.expr_method_call(
                trait_span,
                deserializer,
                cx.ident_of("expect_enum_start"),
                vec!(
                    substr.nonself_args[1],
                    cx.expr_str(trait_span, token::get_ident(substr.type_ident)),
                    cx.expr_vec(trait_span, variants),
                )
            );
            let call = cx.expr_try(trait_span, call);

            let variant = cx.ident_of("i");
            stmts.push(
                cx.stmt_let(
                    trait_span,
                    false,
                    variant,
                    call
                )
            );

            let result = cx.ident_of("result");
            let call = cx.expr_match(
                trait_span,
                cx.expr_ident(trait_span, variant),
                arms
            );
            stmts.push(
                cx.stmt_let(
                    trait_span,
                    false,
                    result,
                    call
                )
            );

            let call = cx.expr_method_call(
                trait_span,
                deserializer,
                cx.ident_of("expect_enum_end"),
                vec!()
            );
            let call = cx.expr_try(trait_span, call);
            stmts.push(cx.stmt_expr(call));

            cx.expr_block(
                cx.block(
                    trait_span,
                    stmts,
                    Some(
                        cx.expr_ok(
                            trait_span,
                            cx.expr_ident(trait_span, result)
                        )
                    )
                )
            )
        }
        _ => cx.bug("expected StaticEnum or StaticStruct in deriving(Deserializable)")
    }
}

/// Create a deserializer for a single enum variant/struct:
/// - `outer_pat_ident` is the name of this enum variant/struct
/// - `getarg` should retrieve the `uint`-th field with name `@str`.
fn deserializable_static_fields(
    cx: &mut ExtCtxt,
    trait_span: Span,
    outer_pat_ident: Ident,
    fields: &StaticFields,
    getarg: |&mut ExtCtxt, Span, token::InternedString| -> Gc<Expr>
) -> Gc<Expr> {
    match *fields {
        Unnamed(ref fields) => {
            if fields.is_empty() {
                cx.expr_ident(trait_span, outer_pat_ident)
            } else {
                let fields = fields.iter().enumerate().map(|(i, &span)| {
                    getarg(
                        cx,
                        span,
                        token::intern_and_get_ident(format!("_field{}", i).as_slice())
                    )
                }).collect();

                cx.expr_call_ident(trait_span, outer_pat_ident, fields)
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

            cx.expr_struct_ident(trait_span, outer_pat_ident, fields)
        }
    }
}
