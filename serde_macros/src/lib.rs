#![feature(custom_derive, plugin, plugin_registrar, rustc_private, unboxed_closures)]
#![plugin(quasi_macros)]

extern crate aster;
extern crate quasi;
extern crate rustc;
extern crate syntax;

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
use syntax::ast_util;
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
    push: &mut FnMut(P<ast::Item>)
) {
    let inline = cx.meta_word(sp, token::InternedString::new("inline"));
    let attrs = vec!(cx.attribute(sp, inline));

    let trait_def = TraitDef {
        span: sp,
        attributes: vec![],
        path: Path::new(vec!["serde", "ser", "Serialize"]),
        additional_bounds: Vec::new(),
        generics: LifetimeBounds::empty(),
        associated_types: vec![],
        methods: vec![
            MethodDef {
                name: "serialize",
                generics: LifetimeBounds {
                    lifetimes: Vec::new(),
                    bounds: vec![
                        ("__S", vec![Path::new(vec!["serde", "ser", "Serializer"])]),
                    ]
                },
                explicit_self: borrowed_explicit_self(),
                args: vec![
                    Ty::Ptr(
                        Box::new(Ty::Literal(Path::new_local("__S"))),
                        Borrowed(None, MutMutable),
                    ),
                ],
                ret_ty: Ty::Literal(
                    Path::new_(
                        vec!("std", "result", "Result"),
                        None,
                        vec![
                            Box::new(Ty::Tuple(vec![])),
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
                    serialize_substructure(a, b, c, item)
                })),
            }
        ]
    };

    trait_def.expand(cx, mitem, item, |item| push(item))
}

fn serialize_substructure(
    cx: &ExtCtxt,
    span: Span,
    substr: &Substructure,
    item: &Item,
) -> P<Expr> {
    let builder = aster::AstBuilder::new().span(span);

    let serializer = substr.nonself_args[0].clone();

    match (&item.node, &*substr.fields) {
        (&ast::ItemStruct(ref struct_def, ref generics), &Struct(ref fields)) => {
            let mut named_fields = vec![];
            let mut unnamed_fields = vec![];

            for field in fields {
                match field.name {
                    Some(name) => { named_fields.push((name, field.span)); }
                    None => { unnamed_fields.push(field.span); }
                }
            }

            match (named_fields.is_empty(), unnamed_fields.is_empty()) {
                (true, true) => {
                    serialize_unit_struct(
                        cx,
                        &builder,
                        serializer,
                        substr.type_ident,
                    )
                }
                (true, false) => {
                    serialize_tuple_struct(
                        cx,
                        &builder,
                        serializer,
                        substr.type_ident,
                        &unnamed_fields,
                        generics,
                    )
                }
                (false, true) => {
                    serialize_struct(
                        cx,
                        &builder,
                        serializer,
                        substr.type_ident,
                        &named_fields,
                        struct_def,
                        generics,
                    )
                }
                (false, false) => {
                    panic!("struct has named and unnamed fields")
                }
            }
        }

        (&ast::ItemEnum(_, ref generics), &EnumMatching(_idx, variant, ref fields)) => {
            serialize_enum(
                cx,
                span,
                &builder,
                serializer,
                substr.type_ident,
                variant,
                &fields,
                generics,
            )
        }

        _ => cx.bug("expected Struct or EnumMatching in derive_serialize")
    }
}

fn serialize_unit_struct(
    cx: &ExtCtxt,
    builder: &aster::AstBuilder,
    serializer: P<Expr>,
    type_ident: Ident
) -> P<Expr> {
    let type_name = builder.expr().str(type_ident);

    quote_expr!(cx, $serializer.visit_named_unit($type_name))
}

fn serialize_tuple_struct(
    cx: &ExtCtxt,
    builder: &aster::AstBuilder,
    serializer: P<Expr>,
    type_ident: Ident,
    fields: &[Span],
    generics: &ast::Generics
) -> P<Expr> {
    let type_name = builder.expr().str(type_ident);
    let len = fields.len();

    let arms: Vec<ast::Arm> = (0 .. fields.len())
        .map(|i| {
            let first = builder.expr().bool(i == 0);
            let expr = builder.expr()
                .tup_field(i)
                .field("value").self_();

            quote_arm!(cx,
                $i => {
                    self.state += 1;
                    let v = try!(serializer.visit_seq_elt($first, &$expr));
                    Ok(Some(v))
                }
            )
        })
        .collect();

    let type_generics = builder.from_generics(generics.clone())
        .strip_bounds()
        .build();

    let visitor_impl_generics = builder.from_generics(generics.clone())
        .add_lifetime_bound("'__a")
        .add_ty_param_bound(
            builder.path().global().ids(&["serde", "ser", "Serialize"]).build()
        )
        .lifetime_name("'__a")
        .build();

    let visitor_generics = builder.from_generics(visitor_impl_generics.clone())
        .strip_bounds()
        .build();

    quote_expr!(cx, {
        struct Visitor $visitor_impl_generics {
            state: usize,
            value: &'__a $type_ident $type_generics,
        }

        impl $visitor_impl_generics ::serde::ser::SeqVisitor for Visitor $visitor_generics {
            #[inline]
            fn visit<S>(&mut self, serializer: &mut S) -> Result<Option<()>, S::Error>
                where S: ::serde::ser::Serializer,
            {
                match self.state {
                    $arms
                    _ => Ok(None),
                }
            }

            #[inline]
            fn len(&self) -> Option<usize> {
                Some($len)
            }
        }

        $serializer.visit_named_seq($type_name, Visitor {
            value: self,
            state: 0,
        })
    })
}

fn serialize_struct(
    cx: &ExtCtxt,
    builder: &aster::AstBuilder,
    serializer: P<Expr>,
    type_ident: Ident,
    fields: &[(Ident, Span)],
    struct_def: &StructDef,
    generics: &ast::Generics
) -> P<Expr> {
    let type_name = builder.expr().str(type_ident);
    let len = fields.len();

    let aliases : Vec<Option<&ast::Lit>> = struct_def.fields.iter()
        .map(field_alias)
        .collect();

    let arms: Vec<ast::Arm> = fields.iter()
        .zip(aliases.iter())
        .enumerate()
        .map(|(i, (&(name, _), alias_lit))| {
            let first = builder.expr().bool(i == 0);

            let expr = match *alias_lit {
                Some(lit) => builder.expr().build_lit(P(lit.clone())),
                None => builder.expr().str(name),
            };

            quote_arm!(cx,
                $i => {
                    self.state += 1;
                    let v = try!(serializer.visit_map_elt($first, $expr, &self.value.$name));
                    Ok(Some(v))
                }
            )
        })
        .collect();

    let type_generics = builder.from_generics(generics.clone())
        .strip_bounds()
        .build();

    let visitor_impl_generics = builder.from_generics(generics.clone())
        .add_lifetime_bound("'__a")
        .add_ty_param_bound(
            builder.path().global().ids(&["serde", "ser", "Serialize"]).build()
        )
        .lifetime_name("'__a")
        .build();

    let visitor_generics = builder.from_generics(visitor_impl_generics.clone())
        .strip_bounds()
        .build();

    quote_expr!(cx, {
        struct Visitor $visitor_impl_generics {
            state: usize,
            value: &'__a $type_ident $type_generics,
        }

        impl $visitor_impl_generics ::serde::ser::MapVisitor for Visitor $visitor_generics {
            #[inline]
            fn visit<S>(&mut self, serializer: &mut S) -> Result<Option<()>, S::Error>
                where S: ::serde::ser::Serializer,
            {
                match self.state {
                    $arms
                    _ => Ok(None),
                }
            }

            #[inline]
            fn len(&self) -> Option<usize> {
                Some($len)
            }
        }

        $serializer.visit_named_map($type_name, Visitor {
            value: self,
            state: 0,
        })
    })
}

fn serialize_enum(
    cx: &ExtCtxt,
    span: Span,
    builder: &aster::AstBuilder,
    serializer: P<Expr>,
    type_ident: Ident,
    variant: &ast::Variant,
    fields: &[FieldInfo],
    generics: &ast::Generics,
) -> P<Expr> {
    let type_name = builder.expr().str(type_ident);
    let variant_name = builder.expr().str(variant.node.name);

    if fields.is_empty() {
        quote_expr!(cx,
            ::serde::ser::Serializer::visit_enum_unit(
                $serializer,
                $type_name,
                $variant_name)
        )
    } else {
        serialize_variant(
            cx,
            span,
            builder,
            serializer,
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
    builder: &aster::AstBuilder,
    serializer: P<ast::Expr>,
    type_name: P<ast::Expr>,
    variant_name: P<ast::Expr>,
    generics: &ast::Generics,
    variant: &ast::Variant,
    fields: &[FieldInfo],
) -> P<Expr> {
    let generics = builder.from_generics(generics.clone())
        .add_lifetime_bound("'__a")
        .add_ty_param_bound(
            builder.path().global().ids(&["serde", "ser", "Serialize"]).build()
        )
        .lifetime_name("'__a")
        .build();

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
                cx.ident_of("MapVisitor"),
                cx.ident_of("visit_enum_map"),
                struct_def.fields.iter()
                    .map(|field| field.node.ty.clone())
                    .collect()
            )
        }
    };

    let value_ty = builder.ty()
        .tuple()
        .with_tys(tys.into_iter().map(|ty| {
            builder.ty()
                .ref_()
                .lifetime("'__a")
                .build_ty(ty)
        }))
        .build();

    let visitor_ident = builder.id("__Visitor");

    let visitor_struct = builder.item().struct_(visitor_ident)
        .generics().with(generics.clone()).build()
        .field("state").usize()
        .field("value").build(value_ty)
        .build();

    let visitor_expr = builder.expr().struct_path(visitor_ident)
        .field("state").usize(0)
        .field("value").tuple()
            .with_exprs(
                fields.iter().map(|field| {
                    builder.expr()
                        .addr_of()
                        .build(field.self_.clone())
                })
            )
            .build()
        .build();

    let mut first = true;

    let visitor_arms: Vec<ast::Arm> = fields.iter()
        .enumerate()
        .map(|(state, field)| {
            let field_expr = builder.expr()
                .tup_field(state)
                .field("value").self_();

            let visit_expr = match field.name {
                Some(real_name) => {
                    let real_name = builder.expr().str(real_name);

                    quote_expr!(cx,
                        ::serde::ser::Serializer::visit_map_elt(
                            serializer,
                            $first,
                            $real_name,
                            $field_expr,
                        )
                    )
                }
                None => {
                    quote_expr!(cx,
                        ::serde::ser::Serializer::visit_seq_elt(
                            serializer,
                            $first,
                            $field_expr,
                        )
                    )
                }
            };

            first = false;

            quote_arm!(cx,
                $state => {
                    self.state += 1;
                    Ok(Some(try!($visit_expr)))
                }
            )
        })
        .collect();

    let trait_path = builder.path()
        .global()
        .ids(&["serde", "ser"]).id(trait_name)
        .build();

    let trait_ref = cx.trait_ref(trait_path);
    let opt_trait_ref = Some(trait_ref);

    let self_ty = builder.ty()
        .path()
        .segment("__Visitor")
            .with_generics(generics.clone())
            .build()
        .build();

    let len = fields.len();
    let impl_ident = ast_util::impl_pretty_name(&opt_trait_ref, Some(&self_ty));

    let methods = vec![
        ast::MethodImplItem(
            quote_method!(cx,
                fn visit<V>(&mut self, serializer: &mut V) -> Result<Option<()>, V::Error>
                    where V: ::serde::ser::Serializer,
                {
                    match self.state {
                        $visitor_arms
                        _ => Ok(None),
                    }
                }
            )
        ),

        ast::MethodImplItem(
            quote_method!(cx,
                fn len(&self) -> Option<usize> {
                    Some($len)
                }
            )
        ),
    ];

    let visitor_impl = cx.item(
        span,
        impl_ident,
        vec![],
        ast::ItemImpl(
            ast::Unsafety::Normal,
            ast::ImplPolarity::Positive,
            generics,
            opt_trait_ref,
            self_ty,
            methods,
        ),
    );

    quote_expr!(cx, {
        $visitor_struct
        $visitor_impl

        ::serde::ser::Serializer::$visitor_method_name(
            $serializer,
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
    push: &mut FnMut(P<ast::Item>)
) {
    let inline = cx.meta_word(sp, token::InternedString::new("inline"));
    let attrs = vec!(cx.attribute(sp, inline));

    let trait_def = TraitDef {
        span: sp,
        attributes: Vec::new(),
        path: Path::new(vec!["serde", "de", "Deserialize"]),
        additional_bounds: Vec::new(),
        generics: LifetimeBounds::empty(),
        associated_types: vec![],
        methods: vec!(
            MethodDef {
                name: "deserialize",
                generics: LifetimeBounds {
                    lifetimes: Vec::new(),
                    bounds: vec![
                        ("__D", vec![Path::new(vec!["serde", "de", "Deserializer"])]),
                    ],
                },
                explicit_self: None,
                args: vec![
                    Ty::Ptr(
                        Box::new(Ty::Literal(Path::new_local("__D"))),
                        Borrowed(None, MutMutable)
                    ),
                ],
                ret_ty: Ty::Literal(
                    Path::new_(
                        vec!["std", "result", "Result"],
                        None,
                        vec![
                            Box::new(Ty::Self_),
                            Box::new(Ty::Literal(Path::new_(vec!["__D", "Error"],
                                                            None,
                                                            vec![],
                                                            false))),
                        ],
                        true
                    )
                ),
                attributes: attrs,
                combine_substructure: combine_substructure(Box::new(|a, b, c| {
                    deserialize_substructure(a, b, c, item)
                })),
            })
    };

    trait_def.expand(cx, mitem, item, |item| push(item))
}

fn deserialize_substructure(
    cx: &ExtCtxt,
    span: Span,
    substr: &Substructure,
    item: &Item,
) -> P<Expr> {
    let builder = aster::AstBuilder::new().span(span);

    let state = substr.nonself_args[0].clone();

    match (&item.node, &*substr.fields) {
        (&ast::ItemStruct(_, ref generics), &StaticStruct(ref struct_def, ref fields)) => {
            deserialize_struct(
                cx,
                span,
                &builder,
                substr.type_ident,
                substr.type_ident,
                cx.path(span, vec![substr.type_ident]),
                fields,
                state,
                struct_def,
                generics,
            )
        }
        (&ast::ItemEnum(_, ref generics), &StaticEnum(ref enum_def, ref fields)) => {
            deserialize_enum(
                cx,
                &builder,
                substr.type_ident,
                &fields,
                state,
                enum_def,
                generics,
            )
        }
        _ => cx.bug("expected StaticEnum or StaticStruct in derive(Deserialize)")
    }
}

fn deserialize_struct(
    cx: &ExtCtxt,
    span: Span,
    builder: &aster::AstBuilder,
    type_ident: Ident,
    struct_ident: Ident,
    struct_path: ast::Path,
    fields: &StaticFields,
    state: P<ast::Expr>,
    struct_def: &StructDef,
    generics: &ast::Generics,
) -> P<ast::Expr> {
    match *fields {
        Unnamed(ref fields) => {
            if fields.is_empty() {
                deserialize_struct_empty_fields(
                    cx,
                    builder,
                    type_ident,
                    struct_ident,
                    struct_path,
                    state)
            } else {
                deserialize_struct_unnamed_fields(
                    cx,
                    builder,
                    type_ident,
                    struct_ident,
                    struct_path,
                    &fields,
                    state,
                    generics,
                )
            }
        }
        Named(ref fields) => {
            deserialize_struct_named_fields(
                cx,
                span,
                builder,
                type_ident,
                struct_ident,
                struct_path,
                &fields,
                state,
                struct_def,
                generics)
        }
    }
}

fn deserialize_struct_empty_fields(
    cx: &ExtCtxt,
    builder: &aster::AstBuilder,
    type_ident: Ident,
    struct_ident: Ident,
    struct_path: ast::Path,
    state: P<ast::Expr>,
) -> P<ast::Expr> {
    let struct_name = builder.expr().str(struct_ident);
    let result = builder.expr().build_path(struct_path);

    quote_expr!(cx, {
        struct __Visitor;

        impl ::serde::de::Visitor for __Visitor {
            type Value = $type_ident;

            #[inline]
            fn visit_unit<E>(&mut self) -> Result<$type_ident, E>
                where E: ::serde::de::Error,
            {
                Ok($result)
            }

            #[inline]
            fn visit_named_unit<
                E: ::serde::de::Error,
            >(&mut self, name: &str) -> Result<$type_ident, E> {
                if name == $struct_name {
                    self.visit_unit()
                } else {
                    Err(::serde::de::Error::syntax_error())
                }
            }


            #[inline]
            fn visit_seq<V>(&mut self, mut visitor: V) -> Result<$type_ident, V::Error>
                where V: ::serde::de::SeqVisitor,
            {
                try!(visitor.end());
                self.visit_unit()
            }
        }

        $state.visit(__Visitor)
    })
}

fn deserialize_struct_unnamed_fields(
    cx: &ExtCtxt,
    builder: &aster::AstBuilder,
    type_ident: Ident,
    struct_ident: Ident,
    struct_path: ast::Path,
    fields: &[Span],
    state: P<ast::Expr>,
    generics: &ast::Generics,
) -> P<ast::Expr> {
    let visitor_impl_generics = builder.from_generics(generics.clone())
        .add_ty_param_bound(
            builder.path().global().ids(&["serde", "de", "Deserialize"]).build()
        )
        .build();

    let field_names: Vec<ast::Ident> = (0 .. fields.len())
        .map(|i| builder.id(&format!("__field{}", i)))
        .collect();

    let visit_seq_expr = declare_visit_seq(
        cx,
        builder,
        struct_path,
        &field_names,
    );

    // Build `__Visitor<A, B, ...>(PhantomData<A>, PhantomData<B>, ...)`
    let (visitor_struct, visitor_expr) = if generics.ty_params.is_empty() {
        (
            builder.item().tuple_struct("__Visitor")
                .build(),
            builder.expr().id("__Visitor"),
        )
    } else {
        (
            builder.item().tuple_struct("__Visitor")
                .generics().with(generics.clone()).build()
                .with_tys(
                    generics.ty_params.iter().map(|ty_param| {
                        builder.ty().phantom_data().id(ty_param.ident)
                    })
                )
                .build(),
            builder.expr().call().id("__Visitor")
                .with_args(
                    generics.ty_params.iter().map(|_| {
                        builder.expr().phantom_data()
                    })
                )
                .build(),
        )
    };

    let struct_name = builder.expr().str(struct_ident);

    let visitor_ty = builder.ty().path()
        .segment("__Visitor").with_generics(generics.clone()).build()
        .build();

    let value_ty = builder.ty().path()
        .segment(type_ident).with_generics(generics.clone()).build()
        .build();

    quote_expr!(cx, {
        $visitor_struct;

        impl $visitor_impl_generics ::serde::de::Visitor for $visitor_ty {
            type Value = $value_ty;

            fn visit_seq<__V>(&mut self, mut visitor: __V) -> Result<$value_ty, __V::Error>
                where __V: ::serde::de::SeqVisitor,
            {
                $visit_seq_expr
            }

            fn visit_named_seq<__V>(&mut self,
                                    name: &str,
                                    visitor: __V) -> Result<$value_ty, __V::Error>
                where __V: ::serde::de::SeqVisitor,
            {
                if name == $struct_name {
                    self.visit_seq(visitor)
                } else {
                    Err(::serde::de::Error::syntax_error())
                }
            }
        }

        $state.visit($visitor_expr)
    })
}

fn declare_visit_seq(
    cx: &ExtCtxt,
    builder: &aster::AstBuilder,
    struct_path: ast::Path,
    field_names: &[Ident],
) -> P<ast::Expr> {
    let let_values: Vec<P<ast::Stmt>> = field_names.iter()
        .map(|name| {
            quote_stmt!(cx,
                let $name = match try!(visitor.visit()) {
                    Some(value) => value,
                    None => {
                        return Err(::serde::de::Error::end_of_stream_error());
                    }
                };
            )
        })
        .collect();

    let result = builder.expr().call()
        .build_path(struct_path)
        .with_args(field_names.iter().map(|name| builder.expr().id(*name)))
        .build();

    quote_expr!(cx, {
        $let_values

        try!(visitor.end());

        Ok($result)
    })
}

fn deserialize_struct_named_fields(
    cx: &ExtCtxt,
    span: Span,
    builder: &aster::AstBuilder,
    type_ident: Ident,
    struct_ident: Ident,
    struct_path: ast::Path,
    fields: &[(Ident, Span)],
    state: P<ast::Expr>,
    struct_def: &StructDef,
    generics: &ast::Generics,
) -> P<ast::Expr> {
    let visitor_impl_generics = builder.from_generics(generics.clone())
        .add_ty_param_bound(
            builder.path().global().ids(&["serde", "de", "Deserialize"]).build()
        )
        .build();

    // Create the field names for the fields.
    let field_names: Vec<ast::Ident> = (0 .. fields.len())
        .map(|i| token::str_to_ident(&format!("__field{}", i)))
        .collect();

    // Build `__Visitor<A, B, ...>(PhantomData<A>, PhantomData<B>, ...)`
    let (visitor_struct, visitor_expr) = if generics.ty_params.is_empty() {
        (
            builder.item().tuple_struct("__Visitor")
                .build(),
            builder.expr().id("__Visitor"),
        )
    } else {
        (
            builder.item().tuple_struct("__Visitor")
                .generics().with(generics.clone()).build()
                .with_tys(
                    generics.ty_params.iter().map(|ty_param| {
                        builder.ty().phantom_data().id(ty_param.ident)
                    })
                )
                .build(),
            builder.expr().call().id("__Visitor")
                .with_args(
                    generics.ty_params.iter().map(|_| {
                        builder.expr().phantom_data()
                    })
                )
                .build(),
        )
    };

    let struct_name = builder.expr().str(struct_ident);

    let visitor_ty = builder.ty().path()
        .segment("__Visitor").with_generics(generics.clone()).build()
        .build();

    let value_ty = builder.ty().path()
        .segment(type_ident).with_generics(generics.clone()).build()
        .build();

    let field_devisitor = declare_map_field_devisitor(
        cx,
        span,
        builder,
        &field_names,
        fields,
        struct_def,
    );

    let visit_map_expr = declare_visit_map(
        cx,
        builder,
        struct_path,
        &field_names,
        fields,
        struct_def
    );

    quote_expr!(cx, {
        $field_devisitor

        $visitor_struct;

        impl $visitor_impl_generics ::serde::de::Visitor for $visitor_ty {
            type Value = $value_ty;

            #[inline]
            fn visit_map<__V>(&mut self, mut visitor: __V) -> Result<$value_ty, __V::Error>
                where __V: ::serde::de::MapVisitor,
            {
                $visit_map_expr
            }

            #[inline]
            fn visit_named_map<__V>(&mut self,
                                    name: &str,
                                    visitor: __V) -> Result<$value_ty, __V::Error>
                where __V: ::serde::de::MapVisitor,
            {
                if name == $struct_name {
                    self.visit_map(visitor)
                } else {
                    Err(::serde::de::Error::syntax_error())
                }
            }
        }

        $state.visit($visitor_expr)
    })
}

fn field_alias(field: &ast::StructField) -> Option<&ast::Lit> {
    field.node.attrs.iter()
        .find(|sa|
              if let MetaItem_::MetaList(ref n, _) = sa.node.value.node {
                  n == &"serde"
              } else {
                  false
              })
        .and_then(|sa|
                  if let MetaItem_::MetaList(_, ref vals) = sa.node.value.node {
                      vals.iter()
                          .fold(None,
                                |v, mi|
                                if let MetaItem_::MetaNameValue(ref n, ref lit) = mi.node {
                                    if n == &"alias" {
                                        Some(lit)
                                    } else {
                                        v
                                    }
                                } else {
                                    v
                                })
                  } else {
                      None
                  })
}

fn declare_map_field_devisitor(
    cx: &ExtCtxt,
    span: Span,
    _builder: &aster::AstBuilder,
    field_names: &[ast::Ident],
    fields: &[(Ident, Span)],
    struct_def: &StructDef,
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

    // Get aliases
    let aliases : Vec<Option<&ast::Lit>> = struct_def.fields.iter()
        .map(field_alias)
        .collect();

    // Match arms to extract a field from a string
    let field_arms: Vec<ast::Arm> = fields.iter()
        .zip(field_names.iter())
        .zip(aliases.iter())
        .map(|((&(name, span), field), alias_lit)| {
            let s = match alias_lit {
                &None => cx.expr_str(span, token::get_ident(name)) ,
                &Some(lit) =>{
                    let lit = (*lit).clone();
                    cx.expr_lit(lit.span, lit.node)
                },
            };
            quote_arm!(cx, $s => Ok(__Field::$field),)})
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
            impl ::serde::de::Visitor for __FieldVisitor {
                type Value = __Field;

                fn visit_str<E>(&mut self, value: &str) -> Result<__Field, E>
                    where E: ::serde::de::Error,
                {
                    match value {
                        $field_arms
                        _ => Err(::serde::de::Error::syntax_error()),
                    }
                }
            }
        ).unwrap(),

        quote_item!(cx,
            impl ::serde::de::Deserialize for __Field {
                #[inline]
                fn deserialize<S>(state: &mut S) -> Result<__Field, S::Error>
                    where S: ::serde::de::Deserializer,
                {
                    state.visit(__FieldVisitor)
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
    builder: &aster::AstBuilder,
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
                    None => try!(visitor.missing_field($name_str)),
                };
            )
        })
        .collect();

    let result = builder.expr().struct_path(struct_path)
        .with_id_exprs(
            fields.iter()
                .zip(field_names.iter())
                .map(|(&(name, _), field)| { 
                    (name, builder.expr().id(field))
                })
        )
        .build();

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
    builder: &aster::AstBuilder,
    type_ident: Ident,
    fields: &[(Ident, Span, StaticFields)],
    state: P<ast::Expr>,
    enum_def: &EnumDef,
    generics: &ast::Generics,
) -> P<ast::Expr> {
    let visitor_impl_generics = builder.from_generics(generics.clone())
        .add_ty_param_bound(
            builder.path().global().ids(&["serde", "de", "Deserialize"]).build()
        )
        .build();

    // Build `__Visitor<A, B, ...>(PhantomData<A>, PhantomData<B>, ...)`
    let (visitor_struct, visitor_expr) = if generics.ty_params.is_empty() {
        (
            builder.item().tuple_struct("__Visitor")
                .build(),
            builder.expr().id("__Visitor"),
        )
    } else {
        (
            builder.item().tuple_struct("__Visitor")
                .generics().with(generics.clone()).build()
                .with_tys(
                    generics.ty_params.iter().map(|ty_param| {
                        builder.ty().phantom_data().id(ty_param.ident)
                    })
                )
                .build(),
            builder.expr().call().id("__Visitor")
                .with_args(
                    generics.ty_params.iter().map(|_| {
                        builder.expr().phantom_data()
                    })
                )
                .build(),
        )
    };

    let visitor_ty = builder.ty().path()
        .segment("__Visitor").with_generics(generics.clone()).build()
        .build();

    let value_ty = builder.ty().path()
        .segment(type_ident).with_generics(generics.clone()).build()
        .build();

    let type_name = builder.expr().str(type_ident);

    // Match arms to extract a variant from a string
    let variant_arms: Vec<ast::Arm> = fields.iter()
        .zip(enum_def.variants.iter())
        .map(|(&(name, span, ref fields), variant_ptr)| {
            let value = deserialize_enum_variant(
                cx,
                span,
                builder,
                type_ident,
                name,
                fields,
                cx.expr_ident(span, cx.ident_of("visitor")),
                variant_ptr,
                &visitor_impl_generics,
                &visitor_ty,
                &visitor_expr,
                &value_ty,
            );

            let s = builder.expr().str(name);
            quote_arm!(cx, $s => $value,)
        })
        .collect();

    quote_expr!(cx, {
        $visitor_struct;

        impl $visitor_impl_generics ::serde::de::Visitor for $visitor_ty {
            type Value = $value_ty;

            fn visit_enum<__V>(&mut self,
                               name: &str,
                               variant: &str,
                               visitor: __V) -> Result<$value_ty, __V::Error>
                where __V: ::serde::de::EnumVisitor,
            {
                if name == $type_name {
                    self.visit_variant(variant, visitor)
                } else {
                    Err(::serde::de::Error::syntax_error())
                }
            }

            fn visit_variant<__V>(&mut self,
                                  name: &str,
                                  mut visitor: __V) -> Result<$value_ty, __V::Error>
                where __V: ::serde::de::EnumVisitor
            {
                match name {
                    $variant_arms
                    _ => Err(::serde::de::Error::syntax_error()),
                }
            }
        }

        $state.visit_enum($visitor_expr)
    })
}

fn deserialize_enum_variant(
    cx: &ExtCtxt,
    span: Span,
    builder: &aster::AstBuilder,
    type_ident: Ident,
    variant_ident: Ident,
    fields: &StaticFields,
    state: P<ast::Expr>,
    variant_ptr: &P<ast::Variant>,
    visitor_impl_generics: &ast::Generics,
    visitor_ty: &P<ast::Ty>,
    visitor_expr: &P<ast::Expr>,
    value_ty: &P<ast::Ty>,
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
                    builder,
                    variant_path,
                    &field_names,
                );

                quote_expr!(cx, {
                    impl $visitor_impl_generics ::serde::de::EnumSeqVisitor for $visitor_ty {
                        type Value = $value_ty;

                        fn visit<
                            V: ::serde::de::SeqVisitor,
                        >(&mut self, mut visitor: V) -> Result<$value_ty, V::Error> {
                            $visit_seq_expr
                        }
                    }

                    $state.visit_seq($visitor_expr)
                })
            }
        }
        Named(ref fields) => {
            // Create the field names for the fields.
            let field_names: Vec<ast::Ident> = (0 .. fields.len())
                .map(|i| token::str_to_ident(&format!("__field{}", i)))
                .collect();

            let field_devisitor = declare_map_field_devisitor(
                cx,
                span,
                builder,
                &field_names,
                fields,
                match variant_ptr.node.kind {
                    ast::VariantKind::StructVariantKind(ref sd) => &*sd,
                    _ => panic!("Mismatched enum types")
                },
            );

            let visit_map_expr = declare_visit_map(
                cx,
                builder,
                variant_path,
                &field_names,
                fields,
                match variant_ptr.node.kind {
                    ast::VariantKind::StructVariantKind(ref sd) => &*sd,
                    _ => panic!("Mismatched enum types")
                },
            );

            quote_expr!(cx, {
                $field_devisitor

                impl $visitor_impl_generics ::serde::de::EnumMapVisitor for $visitor_ty {
                    type Value = $value_ty;

                    fn visit<
                        V: ::serde::de::MapVisitor,
                    >(&mut self, mut visitor: V) -> Result<$value_ty, V::Error> {
                        $visit_map_expr
                    }
                }

                $state.visit_map($visitor_expr)
            })
        }
    }
}
