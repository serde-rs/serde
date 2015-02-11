#![feature(plugin_registrar, quote, unboxed_closures, rustc_private)]

extern crate syntax;
extern crate rustc;

use syntax::ast::{
    Ident,
    MetaItem,
    Item,
    Expr,
    MutMutable,
    //LitNil,
};
use syntax::ast;
use syntax::codemap::{Span, respan};
use syntax::ext::base::{ExtCtxt, Decorator, ItemDecorator};
use syntax::ext::build::AstBuilder;
use syntax::ext::deriving::generic::{
    EnumMatching,
    FieldInfo,
    MethodDef,
    Named,
    StaticFields,
    StaticStruct,
    //StaticEnum,
    Struct,
    Substructure,
    TraitDef,
    Unnamed,
    combine_substructure,
};
use syntax::ext::deriving::generic::ty::{
    Borrowed,
    LifetimeBounds,
    Ty,
    Path,
    //Tuple,
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

fn expand_derive_serialize<>(cx: &mut ExtCtxt,
                                sp: Span,
                                mitem: &MetaItem,
                                item: &Item,
                                mut push: Box<FnMut(P<ast::Item>)>)
{
    let inline = cx.meta_word(sp, token::InternedString::new("inline"));
    let attrs = vec!(cx.attribute(sp, inline));

    let trait_def = TraitDef {
        span: sp,
        attributes: vec!(),
        path: Path::new(vec!["serde2", "ser", "Serialize"]),
        additional_bounds: Vec::new(),
        generics: LifetimeBounds::empty(),
        associated_types: vec![],
        methods: vec![
            MethodDef {
                name: "visit",
                generics: LifetimeBounds {
                    lifetimes: Vec::new(),
                    bounds: vec![
                        ("__V", vec![Path::new(vec!["serde2", "ser", "Visitor"])]),
                    ]
                },
                explicit_self: borrowed_explicit_self(),
                args: vec![
                    Ty::Ptr(
                        Box::new(Ty::Literal(Path::new_local("__V"))),
                        Borrowed(None, MutMutable),
                    ),
                ],
                ret_ty: Ty::Literal(
                    Path::new_(
                        vec!("std", "result", "Result"),
                        None,
                        vec![
                            Box::new(Ty::Literal(Path::new_(vec!["__V", "Value"],
                                                            None,
                                                            vec![],
                                                            false))),
                            Box::new(Ty::Literal(Path::new_(vec!["__V", "Error"],
                                                            None,
                                                            vec![],
                                                            false))),
                        ],
                        true
                    )
                ),
                attributes: attrs,
                combine_substructure: combine_substructure(Box::new(|a, b, c| {
                    serialize_substructure(a, b, c)
                })),
            }
        ]
    };

    trait_def.expand(cx, mitem, item, |item| push(item))
}

fn serialize_substructure(cx: &ExtCtxt, span: Span, substr: &Substructure) -> P<Expr> {
    let visitor = substr.nonself_args[0].clone();

    match *substr.fields {
        Struct(ref fields) => {
            if fields.is_empty() {
                serialize_tuple_struct(cx)
            } else {
                serialize_struct(cx, span, visitor, substr.type_ident, fields)
            }
        }

        EnumMatching(_idx, variant, ref fields) => {
            serialize_enum(cx, span, visitor, substr.type_ident, variant, fields)
        }

        _ => cx.bug("expected Struct or EnumMatching in derive_serialize")
    }
}

fn serialize_tuple_struct(cx: &ExtCtxt) -> P<Expr> {
    // unit structs have no fields and need to return `Ok()`
    quote_expr!(cx, Ok(()))
}

fn serialize_struct(cx: &ExtCtxt,
                    span: Span,
                    visitor: P<Expr>,
                    type_ident: Ident,
                    fields: &Vec<FieldInfo>) -> P<Expr> {

    let type_name = cx.expr_str(
        span,
        token::get_ident(type_ident));
    let len = fields.len();

    let arms: Vec<ast::Arm> = fields.iter()
        .enumerate()
        .map(|(i, &FieldInfo { name, span, .. })| {
            let first = if i == 0 {
                quote_expr!(cx, true)
            } else {
                quote_expr!(cx, false)
            };

            let name = name.unwrap();
            let expr = cx.expr_str(span, token::get_ident(name));

            let i = i as u32;

            quote_arm!(cx,
                $i => {
                    self.state += 1;
                    let v = try!(visitor.visit_map_elt($first, $expr, &self.value.$name));
                    Ok(Some(v))
                }
            )
        })
        .collect();

    quote_expr!(cx, {
        struct Visitor<'a> {
            state: u32,
            value: &'a $type_ident,
        }

        impl<'a> ::serde2::ser::MapVisitor for Visitor<'a> {
            #[inline]
            fn visit<
                V: ::serde2::ser::Visitor,
            >(&mut self, visitor: &mut V) -> Result<Option<V::Value>, V::Error> {
                match self.state {
                    $arms
                    _ => Ok(None),
                }
            }

            #[inline]
            fn size_hint(&self) -> (usize, Option<usize>) {
                let size = $len - (self.state as usize);
                (size, Some(size))
            }
        }

        $visitor.visit_named_map($type_name, Visitor {
            value: self,
            state: 0,
        })
    })
}

fn serialize_enum(cx: &ExtCtxt,
                  span: Span,
                  visitor: P<Expr>,
                  type_ident: Ident,
                  variant: &ast::Variant,
                  fields: &Vec<FieldInfo>) -> P<Expr> {
    let type_name = cx.expr_str(
        span,
        token::get_ident(type_ident)
    );
    let variant_name = cx.expr_str(
        span,
        token::get_ident(variant.node.name)
    );
    let len = fields.len();

    let stmts: Vec<P<ast::Stmt>> = fields.iter()
        .map(|&FieldInfo { ref self_, .. }| {
            quote_stmt!(
                cx,
                try!($visitor.serialize_enum_elt(&$self_))
            )
        })
        .collect();

    quote_expr!(cx, {
        try!($visitor.serialize_enum_start($type_name, $variant_name, $len));
        $stmts
        $visitor.serialize_enum_end()
    })
}

pub fn expand_derive_deserialize(cx: &mut ExtCtxt,
                                 sp: Span,
                                 mitem: &MetaItem,
                                 item: &Item,
                                 mut push: Box<FnMut(P<ast::Item>)>)
{
    let inline = cx.meta_word(sp, token::InternedString::new("inline"));
    let attrs = vec!(cx.attribute(sp, inline));

    let trait_def = TraitDef {
        span: sp,
        attributes: Vec::new(),
        path: Path::new(vec!["serde2", "de", "Deserialize"]),
        additional_bounds: Vec::new(),
        generics: LifetimeBounds::empty(),
        associated_types: vec![],
        methods: vec!(
            MethodDef {
                name: "deserialize",
                generics: LifetimeBounds {
                    lifetimes: Vec::new(),
                    bounds: vec![
                        ("__S", vec![Path::new(vec!["serde2", "de", "Deserializer"])]),
                    ],
                },
                explicit_self: None,
                args: vec![
                    Ty::Ptr(
                        Box::new(Ty::Literal(Path::new_local("__S"))),
                        Borrowed(None, MutMutable)
                    ),
                ],
                ret_ty: Ty::Literal(
                    Path::new_(
                        vec!["std", "result", "Result"],
                        None,
                        vec![
                            Box::new(Ty::Self),
                            Box::new(Ty::Literal(Path::new_(vec!["__S", "Error"],
                                                            None,
                                                            vec![],
                                                            false))),
                        ],
                        true
                    )
                ),
                attributes: attrs,
                combine_substructure: combine_substructure(Box::new(|a, b, c| {
                    deserialize_substructure(a, b, c)
                })),
            })
    };

    trait_def.expand(cx, mitem, item, |item| push(item))
}

fn deserialize_substructure(cx: &ExtCtxt, span: Span, substr: &Substructure) -> P<Expr> {
    let state = substr.nonself_args[0].clone();

    match *substr.fields {
        StaticStruct(_, ref fields) => {
            deserialize_struct(
                cx,
                span,
                substr.type_ident,
                fields,
                state)
        }
        /*
        StaticEnum(_, ref fields) => {
            deserialize_enum(
                cx,
                span,
                substr.type_ident,
                &fields,
                deserializer,
                token)
        }
        */
        _ => cx.bug("expected StaticEnum or StaticStruct in derive(Deserialize)")
    }
}

fn deserialize_struct(
    cx: &ExtCtxt,
    span: Span,
    type_ident: Ident,
    fields: &StaticFields,
    state: P<ast::Expr>,
) -> P<ast::Expr> {
    match *fields {
        Unnamed(ref fields) => {
            deserialize_struct_unnamed_fields(
                cx,
                span,
                type_ident, 
                &fields[],
                state)
        }
        Named(ref fields) => {
            deserialize_struct_named_fields(
                cx,
                span,
                type_ident, 
                &fields[],
                state)
        }
    }
}

fn deserialize_struct_unnamed_fields(
    cx: &ExtCtxt,
    span: Span,
    type_ident: Ident,
    fields: &[Span],
    state: P<ast::Expr>,
) -> P<ast::Expr> {
    let type_name = cx.expr_str(span, token::get_ident(type_ident));

    let field_names: Vec<ast::Ident> = (0 .. fields.len())
        .map(|i| token::str_to_ident(&format!("__field{}", i)))
        .collect();

    let let_values: Vec<P<ast::Stmt>> = field_names.iter()
        .map(|name| {
            quote_stmt!(cx,
                let $name = match try!(visitor.visit()) {
                    Some(value) => value,
                    None => {
                        return Err(::serde2::de::Error::end_of_stream_error());
                    }
                };
            )
        })
        .collect();

    let result = cx.expr_call_ident(
        span,
        type_ident,
        field_names.iter().map(|name| cx.expr_ident(span, *name)).collect());

    quote_expr!(cx, {
        struct __Visitor;

        impl ::serde2::de::Visitor for __Visitor {
            type Value = $type_ident;

            fn visit_seq<
                __V: ::serde2::de::SeqVisitor,
            >(&mut self, mut visitor: __V) -> Result<$type_ident, __V::Error> {
                $let_values

                try!(visitor.end());

                Ok($result)
            }

            fn visit_named_seq<
                __V: ::serde2::de::SeqVisitor,
            >(&mut self, name: &str, visitor: __V) -> Result<$type_ident, __V::Error> {
                if name == $type_name {
                    self.visit_seq(visitor)
                } else {
                    Err(::serde2::de::Error::syntax_error())
                }
            }
        }

        $state.visit(&mut __Visitor)
    })
}

fn deserialize_struct_named_fields(
    cx: &ExtCtxt,
    span: Span,
    type_ident: Ident,
    fields: &[(Ident, Span)],
    state: P<ast::Expr>,
) -> P<ast::Expr> {
    let type_name = cx.expr_str(span, token::get_ident(type_ident));

    // Create the field names for the fields.
    let field_names: Vec<ast::Ident> = (0 .. fields.len())
        .map(|i| token::str_to_ident(&format!("__field{}", i)))
        .collect();

    // Create the field names for the fields.
    let field_variants: Vec<P<ast::Variant>> = field_names.iter()
        .map(|field| {
            P(respan(
                span,
                ast::Variant_ {
                    name: *field,
                    attrs: Vec::new(),
                    kind: ast::TupleVariantKind(Vec::new()),
                    id: ast::DUMMY_NODE_ID,
                    disr_expr: None,
                    vis: ast::Inherited,
                }))
        })
        .collect();

    let field_enum = cx.item_enum(
        span,
        token::str_to_ident("__Field"),
        ast::EnumDef { variants: field_variants });

    // Match arms to extract a field from a string
    let field_arms: Vec<ast::Arm> = fields.iter()
        .zip(field_names.iter())
        .map(|(&(name, span), field)| {
            let s = cx.expr_str(span, token::get_ident(name));
            quote_arm!(cx, $s => Ok(__Field::$field),)
        })
        .collect();

    // Declare each field.
    let let_values: Vec<P<ast::Stmt>> = field_names.iter()
        .map(|field| {
            quote_stmt!(cx, let mut $field = None;)
        })
        .collect();

    // Match arms to extract a value for a field.
    let value_arms: Vec<ast::Arm> = field_names.iter()
        .map(|field| {
            quote_arm!(cx, __Field::$field => {
                $field = Some(try!(visitor.visit_value()));
            })
        })
        .collect();

    let extract_values: Vec<P<ast::Stmt>> = fields.iter()
        .zip(field_names.iter())
        .map(|(&(name, span), field)| {
            let name_str = cx.expr_str(span, token::get_ident(name));
            quote_stmt!(cx,
                let $field = match $field {
                    Some($field) => $field,
                    None => {
                        return Err(::serde2::de::Error::missing_field_error($name_str));
                    }
                };
            )
        })
        .collect();

    let result = cx.expr_struct_ident(
        span,
        type_ident,
        fields.iter()
            .zip(field_names.iter())
            .map(|(&(name, span), field)| {
                cx.field_imm(span, name, cx.expr_ident(span, *field))
            })
            .collect()
    );

    quote_expr!(cx, {
        #[allow(non_camel_case_types)]
        $field_enum

        struct __FieldVisitor;

        impl ::serde2::de::Visitor for __FieldVisitor {
            type Value = __Field;

            fn visit_str<
                E: ::serde2::de::Error,
            >(&mut self, value: &str) -> Result<__Field, E> {
                match value {
                    $field_arms
                    _ => Err(::serde2::de::Error::syntax_error()),
                }
            }
        }

        impl ::serde2::de::Deserialize for __Field {
            #[inline]
            fn deserialize<
                __S: ::serde2::de::Deserializer,
            >(state: &mut __S) -> Result<__Field, __S::Error> {
                state.visit(&mut __FieldVisitor)
            }
        }

        struct __Visitor;

        impl ::serde2::de::Visitor for __Visitor {
            type Value = $type_ident;

            fn visit_map<
                __V: ::serde2::de::MapVisitor,
            >(&mut self, mut visitor: __V) -> Result<$type_ident, __V::Error> {
                $let_values

                while let Some(key) = try!(visitor.visit_key()) {
                    match key {
                        $value_arms
                    }
                }

                $extract_values
                Ok($result)
            }

            fn visit_named_map<
                __V: ::serde2::de::MapVisitor,
            >(&mut self, name: &str, visitor: __V) -> Result<$type_ident, __V::Error> {
                if name == $type_name {
                    self.visit_map(visitor)
                } else {
                    Err(::serde2::de::Error::syntax_error())
                }
            }
        }

        $state.visit(&mut __Visitor)
    })
}

/*
fn deserialize_struct(
    cx: &ExtCtxt,
    span: Span,
    type_ident: Ident,
    fields: &StaticFields,
    deserializer: P<ast::Expr>,
    token: P<ast::Expr>
) -> P<ast::Expr> {
    /*
    let struct_block = deserialize_struct_from_struct(
        cx,
        span,
        type_ident,
        fields,
        deserializer
    );
    */

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
            ::serde2::de::StructStart(_, _) => $struct_block,
            ::serde2::de::MapStart(_) => $map_block,
            token => {
                let expected_tokens = [
                    ::serde2::de::StructStartKind,
                    ::serde2::de::MapStartKind,
                ];
                Err($deserializer.syntax_error(token, expected_tokens))
            }
        }
    )
}

/*
fn deserialize_struct_from_struct(
    cx: &ExtCtxt,
    span: Span,
    type_ident: Ident,
    fields: &StaticFields,
    deserializer: P<ast::Expr>
) -> P<ast::Expr> {
    let expect_struct_field = cx.ident_of("expect_struct_field");

    let call = deserialize_static_fields(
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
*/

fn deserialize_struct_from_map(
    cx: &ExtCtxt,
    span: Span,
    type_ident: Ident,
    fields: &StaticFields,
    deserializer: P<ast::Expr>
) -> P<ast::Expr> {
    let fields = match *fields {
        Unnamed(_) => panic!(),
        Named(ref fields) => &fields[],
    };

    // Declare each field.
    let let_fields: Vec<P<ast::Stmt>> = fields.iter()
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
                        try!(::serde2::de::Deserialize::deserialize($deserializer))
                    );
                    continue;
                })
        })
        .collect();

    let extract_fields: Vec<P<ast::Stmt>> = fields.iter()
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
                ::serde2::de::End => { break; }
                token => token,
            };

            {
                let key = match token {
                    ::serde2::de::Str(s) => s,
                    ::serde2::de::String(ref s) => &s,
                    token => {
                        let expected_tokens = [
                            ::serde2::de::StrKind,
                            ::serde2::de::StringKind,
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
    deserializer: P<ast::Expr>,
    token: P<ast::Expr>
) -> P<ast::Expr> {
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
            let call = deserialize_static_fields(
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
/// - `getarg` should retrieve the `u32`-th field with name `&str`.
fn deserialize_static_fields(
    cx: &ExtCtxt,
    span: Span,
    outer_pat_ident: Ident,
    fields: &StaticFields,
    getarg: |&ExtCtxt, Span, token::InternedString| -> P<Expr>
) -> P<Expr> {
    match *fields {
        Unnamed(ref fields) => {
            if fields.is_empty() {
                cx.expr_ident(span, outer_pat_ident)
            } else {
                let fields = fields.iter().enumerate().map(|(i, &span)| {
                    getarg(
                        cx,
                        span,
                        token::intern_and_get_ident(&format!("_field{}", i))
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
*/
