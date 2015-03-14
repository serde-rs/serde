use syntax::ast::{
    Ident,
    MetaItem,
    Item,
    Expr,
    MutMutable,
    StructDef,
};
use syntax::ast;
use syntax::codemap::Span;
use syntax::ext::base::{ExtCtxt, ItemDecorator};
use syntax::ext::build::AstBuilder;
use syntax::ext::deriving::generic::{
    EnumMatching,
    FieldInfo,
    MethodDef,
    Struct,
    Substructure,
    TraitDef,
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

use aster;

use field::field_alias;

pub fn expand_derive_serialize(
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
            let mut unnamed_fields = 0;

            for field in fields {
                match field.name {
                    Some(name) => { named_fields.push(name); }
                    None => { unnamed_fields += 1; }
                }
            }

            match (named_fields.is_empty(), unnamed_fields == 0) {
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
                        unnamed_fields,
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
                    cx.bug("struct has named and unnamed fields")
                }
            }
        }

        (&ast::ItemEnum(_, ref generics), &EnumMatching(_idx, variant, ref fields)) => {
            serialize_enum(
                cx,
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
    fields: usize,
    generics: &ast::Generics
) -> P<Expr> {
    let value_ty = builder.ty()
        .ref_()
            .lifetime("'__a")
            .ty().path()
                .segment(type_ident).with_generics(generics.clone()).build()
                .build();

    let (visitor_struct, visitor_impl) = serialize_tuple_struct_visitor(
        cx,
        builder,
        value_ty,
        fields,
        generics,
    );

    let type_name = builder.expr().str(type_ident);

    quote_expr!(cx, {
        $visitor_struct
        $visitor_impl
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
    fields: &[Ident],
    struct_def: &StructDef,
    generics: &ast::Generics
) -> P<Expr> {
    let value_ty = builder.ty()
        .ref_()
            .lifetime("'__a")
            .ty().path()
                .segment(type_ident).with_generics(generics.clone()).build()
                .build();

    let (visitor_struct, visitor_impl) = serialize_struct_visitor(
        cx,
        builder,
        value_ty,
        struct_def,
        generics,
        fields.iter().map(|field| quote_expr!(cx, &self.value.$field)),
    );

    let type_name = builder.expr().str(type_ident);

    quote_expr!(cx, {
        $visitor_struct
        $visitor_impl
        $serializer.visit_named_map($type_name, Visitor {
            value: self,
            state: 0,
        })
    })
}

fn serialize_enum(
    cx: &ExtCtxt,
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
        match variant.node.kind {
            ast::TupleVariantKind(ref args) => {
                serialize_tuple_variant(
                    cx,
                    builder,
                    serializer,
                    type_name,
                    variant_name,
                    generics,
                    args,
                    fields,
                )
            }
            ast::StructVariantKind(ref struct_def) => {
                serialize_struct_variant(
                    cx,
                    builder,
                    serializer,
                    type_name,
                    variant_name,
                    generics,
                    struct_def,
                    fields,
                )
            }
        }
    }
}

fn serialize_tuple_variant(
    cx: &ExtCtxt,
    builder: &aster::AstBuilder,
    serializer: P<Expr>,
    type_name: P<ast::Expr>,
    variant_name: P<ast::Expr>,
    generics: &ast::Generics,
    args: &[ast::VariantArg],
    fields: &[FieldInfo],
) -> P<Expr> {
    let value_ty = builder.ty().tuple()
        .with_tys(
            args.iter().map(|arg| {
                builder.ty()
                    .ref_()
                    .lifetime("'__a")
                    .build_ty(arg.ty.clone())
            })
        )
        .build();

    let value_expr = builder.expr().tuple()
        .with_exprs(
            fields.iter().map(|field| {
                builder.expr()
                    .addr_of()
                    .build(field.self_.clone())
            })
        )
        .build();

    let (visitor_struct, visitor_impl) = serialize_tuple_struct_visitor(
        cx,
        builder,
        value_ty,
        args.len(),
        generics,
    );

    quote_expr!(cx, {
        $visitor_struct
        $visitor_impl
        $serializer.visit_enum_seq($type_name, $variant_name, Visitor {
            value: $value_expr,
            state: 0,
        })
    })
}

fn serialize_struct_variant(
    cx: &ExtCtxt,
    builder: &aster::AstBuilder,
    serializer: P<Expr>,
    type_name: P<ast::Expr>,
    variant_name: P<ast::Expr>,
    generics: &ast::Generics,
    struct_def: &ast::StructDef,
    fields: &[FieldInfo],
) -> P<Expr> {
    let value_ty = builder.ty().tuple()
        .with_tys(
            struct_def.fields.iter().map(|field| {
                builder.ty()
                    .ref_()
                    .lifetime("'__a")
                    .build_ty(field.node.ty.clone())
            })
        )
        .build();

    let value_expr = builder.expr().tuple()
        .with_exprs(
            fields.iter().map(|field| {
                builder.expr()
                    .addr_of()
                    .build(field.self_.clone())
            })
        )
        .build();

    let (visitor_struct, visitor_impl) = serialize_struct_visitor(
        cx,
        builder,
        value_ty,
        struct_def,
        generics,
        (0 .. fields.len()).map(|i| {
            builder.expr()
                .tup_field(i)
                .field("value").self_()
        })
    );

    quote_expr!(cx, {
        $visitor_struct
        $visitor_impl
        $serializer.visit_enum_map($type_name, $variant_name, Visitor {
            value: $value_expr,
            state: 0,
        })
    })
}

fn serialize_tuple_struct_visitor(
    cx: &ExtCtxt,
    builder: &aster::AstBuilder,
    value_ty: P<ast::Ty>,
    fields: usize,
    generics: &ast::Generics
) -> (P<ast::Item>, P<ast::Item>) {
    let arms: Vec<ast::Arm> = (0 .. fields)
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

    let visitor_impl_generics = builder.from_generics(generics.clone())
        .add_lifetime_bound("'__a")
        .add_ty_param_bound(
            builder.path().global().ids(&["serde", "ser", "Serialize"]).build()
        )
        .lifetime_name("'__a")
        .build();

    let where_clause = &visitor_impl_generics.where_clause;

    let visitor_generics = builder.from_generics(visitor_impl_generics.clone())
        .strip_bounds()
        .build();

    (
        quote_item!(cx,
            struct Visitor $visitor_impl_generics $where_clause {
                state: usize,
                value: $value_ty,
            }
        ).unwrap(),

        quote_item!(cx,
            impl $visitor_impl_generics ::serde::ser::SeqVisitor
            for Visitor $visitor_generics
            $where_clause {
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
                    Some($fields)
                }
            }
        ).unwrap(),
    )
}

fn serialize_struct_visitor<I>(
    cx: &ExtCtxt,
    builder: &aster::AstBuilder,
    value_ty: P<ast::Ty>,
    struct_def: &StructDef,
    generics: &ast::Generics,
    value_exprs: I,
) -> (P<ast::Item>, P<ast::Item>)
    where I: Iterator<Item=P<ast::Expr>>,
{
    let len = struct_def.fields.len();

    let key_exprs = struct_def.fields.iter()
        .map(|field| {
            match field_alias(field) {
                Some(lit) => builder.expr().build_lit(P(lit.clone())),
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
        });

    let arms: Vec<ast::Arm> = key_exprs
        .zip(value_exprs)
        .enumerate()
        .map(|(i, (key_expr, value_expr))| {
            let first = i == 0;

            quote_arm!(cx,
                $i => {
                    self.state += 1;
                    Ok(
                        Some(
                            try!(
                                serializer.visit_map_elt(
                                    $first,
                                    $key_expr,
                                    $value_expr,
                                )
                            )
                        )
                    )
                }
            )
        })
        .collect();

    let visitor_impl_generics = builder.from_generics(generics.clone())
        .add_lifetime_bound("'__a")
        .add_ty_param_bound(
            builder.path().global().ids(&["serde", "ser", "Serialize"]).build()
        )
        .lifetime_name("'__a")
        .build();

    let where_clause = &visitor_impl_generics.where_clause;

    let visitor_generics = builder.from_generics(visitor_impl_generics.clone())
        .strip_bounds()
        .build();

    (
        quote_item!(cx,
            struct Visitor $visitor_impl_generics $where_clause {
                state: usize,
                value: $value_ty,
            }
        ).unwrap(),

        quote_item!(cx,
            impl $visitor_impl_generics
            ::serde::ser::MapVisitor
            for Visitor $visitor_generics
            $where_clause {
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
        ).unwrap(),
    )
}
