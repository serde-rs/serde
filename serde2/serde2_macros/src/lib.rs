#![feature(plugin_registrar, quote, unboxed_closures, rustc_private)]

extern crate syntax;
extern crate rustc;

use syntax::ast::{
    Ident,
    MetaItem,
    MetaItem_,
    Item,
    Expr,
    MutMutable,
    StructDef,
    EnumDef,
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
    Ty,
    Path,
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

fn expand_derive_serialize(
    cx: &mut ExtCtxt,
    sp: Span,
    mitem: &MetaItem,
    item: &Item,
    push: &mut FnMut(P<ast::Item>))
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
                    serialize_substructure(a, b, c, item)
                })),
            }
        ]
    };

    trait_def.expand(cx, mitem, item, |item| push(item))
}

fn serialize_substructure(cx: &ExtCtxt,
                          span: Span,
                          substr: &Substructure,
                          item: &Item) -> P<Expr> {
    let visitor = substr.nonself_args[0].clone();

    match (&item.node, &*substr.fields) {
        (&ast::ItemStruct(..), &Struct(ref fields)) => {
            if fields.is_empty() {
                serialize_tuple_struct(cx)
            } else {
                serialize_struct(cx, span, visitor, substr.type_ident, fields)
            }
        }

        (&ast::ItemEnum(_, ref generics), &EnumMatching(_idx, variant, ref fields)) => {
            serialize_enum(cx,
                           span,
                           visitor,
                           substr.type_ident,
                           variant,
                           &fields[],
                           generics)
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
                    fields: &[FieldInfo]) -> P<Expr> {
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




fn serialize_enum(
    cx: &ExtCtxt,
    span: Span,
    visitor: P<Expr>,
    type_ident: Ident,
    variant: &ast::Variant,
    fields: &[FieldInfo],
    generics: &ast::Generics,
) -> P<Expr> {
    let type_name = cx.expr_str(span, token::get_ident(type_ident));
    let variant_ident = variant.node.name;
    let variant_name = cx.expr_str(span, token::get_ident(variant_ident));

    if fields.is_empty() {
        quote_expr!(cx,
            ::serde2::ser::Visitor::visit_enum_unit(
                $visitor,
                $type_name,
                $variant_name)
        )
    } else {
        serialize_variant(
                cx,
                span,
                visitor,
                type_name,
                variant_name,
                generics,
                variant,
                fields)
    }
}


fn serialize_variant(
    cx: &ExtCtxt,
    span: Span,
    visitor: P<ast::Expr>,
    type_name: P<ast::Expr>,
    variant_name: P<ast::Expr>,
    generics: &ast::Generics,
    variant: &ast::Variant,
    fields: &[FieldInfo],
) -> P<Expr> {
    // We'll take a reference to the values in the variant, so we need a new lifetime.
    let lifetimes = generics.lifetimes.iter()
        .map(|lifetime_def| lifetime_def.lifetime.clone())
        .collect();

    let mut generics = generics.clone();
    generics.lifetimes.push(
        cx.lifetime_def(
            span,
            cx.name_of("'__a"),
            lifetimes
        )
    );

    let (
        trait_name,
        visitor_method_name,
        tys,
    ): (Ident, Ident, Vec<P<ast::Ty>>) = match variant.node.kind {
        ast::TupleVariantKind(ref args) => {
            (
                cx.ident_of("SeqVisitor"),
                cx.ident_of("visit_enum_seq"),
                args.iter()
                    .map(|arg| arg.ty.clone())
                    .collect()
            )
        }

        ast::StructVariantKind(ref struct_def) => {
            (
                cx.ident_of("SeqVisitor"),
                cx.ident_of("visit_enum_seq"),
                struct_def.fields.iter()
                    .map(|field| field.node.ty.clone())
                    .collect()
            )
        }
    };

    let len = fields.len();

    let visitor_field_names: Vec<ast::Ident> = (0 .. len)
        .map(|i| token::str_to_ident(&format!("field{}", i)))
        .collect();

    let mut visitor_fields = vec![
        respan(
            span,
            ast::StructField_ {
                kind: ast::NamedField(
                    cx.ident_of("state"),
                    ast::Visibility::Inherited,
                ),
                id: ast::DUMMY_NODE_ID,
                ty: cx.ty_ident(span, cx.ident_of("u32")),
                attrs: Vec::new(),
            }
        ),
    ];

    visitor_fields.extend(
        visitor_field_names.iter()
            .zip(tys.iter())
            .map(|(name, ty)| {
                respan(
                    span,
                    ast::StructField_ {
                        kind: ast::NamedField(
                            *name,
                            ast::Visibility::Inherited,
                        ),
                        id: ast::DUMMY_NODE_ID,
                        ty: cx.ty_rptr(
                            span,
                            ty.clone(),
                            Some(cx.lifetime(span, cx.name_of("'__a"))),
                            ast::MutImmutable,
                        ),
                        attrs: Vec::new(),
                    }
                )
            })
    );

    let visitor_ident = cx.ident_of("__Visitor");

    let visitor_struct = cx.item_struct_poly(
        span,
        visitor_ident,
        ast::StructDef {
            fields: visitor_fields,
            ctor_id: None,
        },
        generics,
    );

    let mut visitor_field_exprs = vec![
        cx.field_imm(
            span,
            cx.ident_of("state"),
            quote_expr!(cx, 0),
        ),
    ];

    visitor_field_exprs.extend(
        visitor_field_names.iter()
            .zip(fields.iter())
            .map(|(name, field)| {
                let e = cx.expr_addr_of(
                    span,
                    field.self_.clone(),
                );

                cx.field_imm(span, *name, e)
            })
    );

    let visitor_expr = cx.expr_struct(
        span,
        cx.path_ident(span, visitor_ident),
        visitor_field_exprs);

    let mut first = true;

    let visitor_arms: Vec<ast::Arm> = visitor_field_names.iter()
        .zip(fields.iter())
        .enumerate()
        .map(|(state, (name, field))| {
            let field_expr = cx.expr_field_access(
                span,
                cx.expr_self(span),
                *name,
            );

            let visit_expr = match field.name {
                Some(real_name) => {
                    let real_name = cx.expr_str(span, token::get_ident(real_name));
                    quote_expr!(cx,
                        ::serde2::ser::Visitor::visit_map_elt(
                            visitor,
                            $first,
                            $real_name,
                            $field_expr,
                        )
                    )
                }
                None => {
                    quote_expr!(cx,
                        ::serde2::ser::Visitor::visit_seq_elt(
                            visitor,
                            $first,
                            $field_expr,
                        )
                    )
                }
            };

            first = false;

            let state = state as u32;

            quote_arm!(cx,
                $state => {
                    self.state += 1;
                    Ok(Some(try!($visit_expr)))
                }
            )
        })
        .collect();

    quote_expr!(cx, {
        $visitor_struct

        impl<'__a> ::serde2::ser::$trait_name for __Visitor<'__a> {
            fn visit<
                V: ::serde2::ser::Visitor,
            >(&mut self, visitor: &mut V) -> Result<Option<V::Value>, V::Error> {
                match self.state {
                    $visitor_arms
                    _ => Ok(None),
                }
            }

            fn size_hint(&self) -> (usize, Option<usize>) {
                ($len - self.state as usize, Some($len - self.state as usize))
            }
        }

        ::serde2::ser::Visitor::$visitor_method_name(
            $visitor,
            $type_name,
            $variant_name,
            $visitor_expr,
        )
    })
}

pub fn expand_derive_deserialize(
    cx: &mut ExtCtxt,
    sp: Span,
    mitem: &MetaItem,
    item: &Item,
    push: &mut FnMut(P<ast::Item>))
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
                            Box::new(Ty::Self_),
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
        StaticStruct(ref struct_def, ref fields) => {
            deserialize_struct(
                cx,
                span,
                substr.type_ident,
                substr.type_ident,
                cx.path(span, vec![substr.type_ident]),
                fields,
                state,
                struct_def)
        }
        StaticEnum(ref enum_def, ref fields) => {
            deserialize_enum(
                cx,
                span,
                substr.type_ident,
                &fields,
                state,
                enum_def)
        }
        _ => cx.bug("expected StaticEnum or StaticStruct in derive(Deserialize)")
    }
}

fn deserialize_struct(
    cx: &ExtCtxt,
    span: Span,
    type_ident: Ident,
    struct_ident: Ident,
    struct_path: ast::Path,
    fields: &StaticFields,
    state: P<ast::Expr>,
    struct_def: &StructDef
) -> P<ast::Expr> {
    match *fields {
        Unnamed(ref fields) => {
            if fields.is_empty() {
                deserialize_struct_empty_fields(
                    cx,
                    span,
                    type_ident,
                    struct_ident,
                    struct_path,
                    state)
            } else {
                deserialize_struct_unnamed_fields(
                    cx,
                    span,
                    type_ident,
                    struct_ident,
                    struct_path,
                    &fields[],
                    state)
            }
        }
        Named(ref fields) => {
            deserialize_struct_named_fields(
                cx,
                span,
                type_ident,
                struct_ident,
                struct_path,
                &fields[],
                state,
                struct_def)
        }
    }
}

fn deserialize_struct_empty_fields(
    cx: &ExtCtxt,
    span: Span,
    type_ident: Ident,
    struct_ident: Ident,
    struct_path: ast::Path,
    state: P<ast::Expr>,
) -> P<ast::Expr> {
    let struct_name = cx.expr_str(span, token::get_ident(struct_ident));

    let result = cx.expr_path(struct_path);

    quote_expr!(cx, {
        struct __Visitor;

        impl ::serde2::de::Visitor for __Visitor {
            type Value = $type_ident;

            #[inline]
            fn visit_unit<
                E: ::serde2::de::Error,
            >(&mut self) -> Result<$type_ident, E> {
                Ok($result)
            }

            #[inline]
            fn visit_named_unit<
                E: ::serde2::de::Error,
            >(&mut self, name: &str) -> Result<$type_ident, E> {
                if name == $struct_name {
                    self.visit_unit()
                } else {
                    Err(::serde2::de::Error::syntax_error())
                }
            }
        }

        $state.visit(&mut __Visitor)
    })
}

fn deserialize_struct_unnamed_fields(
    cx: &ExtCtxt,
    span: Span,
    type_ident: Ident,
    struct_ident: Ident,
    struct_path: ast::Path,
    fields: &[Span],
    state: P<ast::Expr>,
) -> P<ast::Expr> {
    let struct_name = cx.expr_str(span, token::get_ident(struct_ident));

    let field_names: Vec<ast::Ident> = (0 .. fields.len())
        .map(|i| token::str_to_ident(&format!("__field{}", i)))
        .collect();

    let visit_seq_expr = declare_visit_seq(
        cx,
        span,
        struct_path,
        &field_names[],
    );

    quote_expr!(cx, {
        struct __Visitor;

        impl ::serde2::de::Visitor for __Visitor {
            type Value = $type_ident;

            fn visit_seq<
                __V: ::serde2::de::SeqVisitor,
            >(&mut self, mut visitor: __V) -> Result<$type_ident, __V::Error> {
                $visit_seq_expr
            }

            fn visit_named_seq<
                __V: ::serde2::de::SeqVisitor,
            >(&mut self, name: &str, visitor: __V) -> Result<$type_ident, __V::Error> {
                if name == $struct_name {
                    self.visit_seq(visitor)
                } else {
                    Err(::serde2::de::Error::syntax_error())
                }
            }
        }

        $state.visit(&mut __Visitor)
    })
}

fn declare_visit_seq(
    cx: &ExtCtxt,
    span: Span,
    struct_path: ast::Path,
    field_names: &[Ident],
) -> P<ast::Expr> {
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

    let result = cx.expr_call(
        span,
        cx.expr_path(struct_path),
        field_names.iter().map(|name| cx.expr_ident(span, *name)).collect());

    quote_expr!(cx, {
        $let_values

        try!(visitor.end());

        Ok($result)
    })
}

fn deserialize_struct_named_fields(
    cx: &ExtCtxt,
    span: Span,
    type_ident: Ident,
    struct_ident: Ident,
    struct_path: ast::Path,
    fields: &[(Ident, Span)],
    state: P<ast::Expr>,
    struct_def: &StructDef,
) -> P<ast::Expr> {
    let struct_name = cx.expr_str(span, token::get_ident(struct_ident));

    // Create the field names for the fields.
    let field_names: Vec<ast::Ident> = (0 .. fields.len())
        .map(|i| token::str_to_ident(&format!("__field{}", i)))
        .collect();

    let field_deserializer = declare_map_field_deserializer(
        cx,
        span,
        &field_names[],
        fields,
    );

    let visit_map_expr = declare_visit_map(
        cx,
        span,
        struct_path,
        &field_names[],
        fields,
        struct_def
    );

    quote_expr!(cx, {
        $field_deserializer

        struct __Visitor;

        impl ::serde2::de::Visitor for __Visitor {
            type Value = $type_ident;

            #[inline]
            fn visit_map<
                __V: ::serde2::de::MapVisitor,
            >(&mut self, mut visitor: __V) -> Result<$type_ident, __V::Error> {
                $visit_map_expr
            }

            #[inline]
            fn visit_named_map<
                __V: ::serde2::de::MapVisitor,
            >(&mut self, name: &str, visitor: __V) -> Result<$type_ident, __V::Error> {
                if name == $struct_name {
                    self.visit_map(visitor)
                } else {
                    Err(::serde2::de::Error::syntax_error())
                }
            }
        }

        $state.visit(&mut __Visitor)
    })
}

fn declare_map_field_deserializer(
    cx: &ExtCtxt,
    span: Span,
    field_names: &[ast::Ident],
    fields: &[(Ident, Span)],
) -> Vec<P<ast::Item>> {
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

    vec![
        quote_item!(cx,
            #[allow(non_camel_case_types)]
            $field_enum
        ).unwrap(),

        quote_item!(cx,
            struct __FieldVisitor;
        ).unwrap(),

        quote_item!(cx,
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
        ).unwrap(),

        quote_item!(cx,
            impl ::serde2::de::Deserialize for __Field {
                #[inline]
                fn deserialize<
                    __S: ::serde2::de::Deserializer,
                >(state: &mut __S) -> Result<__Field, __S::Error> {
                    state.visit(&mut __FieldVisitor)
                }
            }
        ).unwrap(),
    ]
}


fn default_value(field: &ast::StructField) -> bool {
    field.node.attrs.iter()
        .any(|sa|
             if let MetaItem_::MetaList(ref n, ref vals) = sa.node.value.node {
                 if n == &"serde" {
                     vals.iter()
                         .map(|mi|
                              if let MetaItem_::MetaWord(ref n) = mi.node {
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
             })
}

fn declare_visit_map(
    cx: &ExtCtxt,
    span: Span,
    struct_path: ast::Path,
    field_names: &[Ident],
    fields: &[(Ident, Span)],
    struct_def: &StructDef,
) -> P<ast::Expr> {

    // Declare each field.
    let let_values: Vec<P<ast::Stmt>> = field_names.iter()
        .zip(struct_def.fields.iter())
        .map(|(field, sf)| {
            if default_value(sf) {
                quote_stmt!(
                    cx,
                    let mut $field = Some(::std::default::Default::default());)
            } else {
                quote_stmt!(cx, let mut $field = None;)
            }
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

    let result = cx.expr_struct(
        span,
        struct_path,
        fields.iter()
            .zip(field_names.iter())
            .map(|(&(name, span), field)| {
                cx.field_imm(span, name, cx.expr_ident(span, *field))
            })
            .collect()
    );

    quote_expr!(cx, {
        $let_values

        while let Some(key) = try!(visitor.visit_key()) {
            match key {
                $value_arms
            }
        }

        $extract_values
        Ok($result)
    })
}

fn deserialize_enum(
    cx: &ExtCtxt,
    span: Span,
    type_ident: Ident,
    fields: &[(Ident, Span, StaticFields)],
    state: P<ast::Expr>,
    enum_def: &EnumDef,
) -> P<ast::Expr> {
    let type_name = cx.expr_str(span, token::get_ident(type_ident));

    // Match arms to extract a variant from a string
    let variant_arms: Vec<ast::Arm> = fields.iter()
        .zip(enum_def.variants.iter())
        .map(|(&(name, span, ref fields), variant_ptr)| {
            let value = deserialize_enum_variant(
                cx,
                span,
                type_ident,
                name,
                fields,
                cx.expr_ident(span, cx.ident_of("visitor")),
                variant_ptr,
            );

            let s = cx.expr_str(span, token::get_ident(name));
            quote_arm!(cx, $s => $value,)
        })
        .collect();

    quote_expr!(cx, {
        struct __Visitor;

        impl ::serde2::de::Visitor for __Visitor {
            type Value = $type_ident;

            fn visit_enum<
                __V: ::serde2::de::EnumVisitor,
            >(&mut self, name: &str, variant: &str, mut visitor: __V) -> Result<$type_ident, __V::Error> {
                if name == $type_name {
                    self.visit_variant(variant, visitor)
                } else {
                    Err(::serde2::de::Error::syntax_error())
                }
            }

            fn visit_variant<
                __V: ::serde2::de::EnumVisitor,
            >(&mut self, name: &str, mut visitor: __V) -> Result<$type_ident, __V::Error> {
                match name {
                    $variant_arms
                    _ => Err(::serde2::de::Error::syntax_error()),
                }
            }
        }

        $state.visit(&mut __Visitor)
    })
}

fn deserialize_enum_variant(
    cx: &ExtCtxt,
    span: Span,
    type_ident: Ident,
    variant_ident: Ident,
    fields: &StaticFields,
    state: P<ast::Expr>,
    variant_ptr: &P<ast::Variant>,
) -> P<ast::Expr> {
    let variant_path = cx.path(span, vec![type_ident, variant_ident]);

    match *fields {
        Unnamed(ref fields) => {
            if fields.is_empty() {
                let result = cx.expr_path(variant_path);

                quote_expr!(cx, {
                    try!($state.visit_unit());
                    Ok($result)
                })
            } else {
                // Create the field names for the fields.
                let field_names: Vec<ast::Ident> = (0 .. fields.len())
                    .map(|i| token::str_to_ident(&format!("__field{}", i)))
                    .collect();

                let visit_seq_expr = declare_visit_seq(
                    cx,
                    span,
                    variant_path,
                    &field_names[],
                );

                quote_expr!(cx, {
                    struct __Visitor;

                    impl ::serde2::de::EnumSeqVisitor for __Visitor {
                        type Value = $type_ident;

                        fn visit<
                            V: ::serde2::de::SeqVisitor,
                        >(&mut self, mut visitor: V) -> Result<$type_ident, V::Error> {
                            $visit_seq_expr
                        }
                    }

                    $state.visit_seq(&mut __Visitor)
                })
            }
        }
        Named(ref fields) => {
            // Create the field names for the fields.
            let field_names: Vec<ast::Ident> = (0 .. fields.len())
                .map(|i| token::str_to_ident(&format!("__field{}", i)))
                .collect();

            let field_deserializer = declare_map_field_deserializer(
                cx,
                span,
                &field_names[],
                fields,
            );

            let visit_map_expr = declare_visit_map(
                cx,
                span,
                variant_path,
                &field_names[],
                fields,
                match variant_ptr.node.kind {
                    ast::VariantKind::StructVariantKind(ref sd) => &*sd,
                    _ => panic!("Mismatched enum types")
                },
            );

            quote_expr!(cx, {
                $field_deserializer

                struct __Visitor;

                impl ::serde2::de::EnumMapVisitor for __Visitor {
                    type Value = $type_ident;

                    fn visit<
                        V: ::serde2::de::MapVisitor,
                    >(&mut self, mut visitor: V) -> Result<$type_ident, V::Error> {
                        $visit_map_expr
                    }
                }

                $state.visit_map(&mut __Visitor)
            })
        }
    }
}
