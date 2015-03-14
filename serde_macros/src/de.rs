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
use syntax::ext::base::{ExtCtxt, ItemDecorator};
use syntax::ext::build::AstBuilder;
use syntax::ext::deriving::generic::{
    MethodDef,
    Named,
    StaticFields,
    StaticStruct,
    StaticEnum,
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
};
use syntax::parse::token;
use syntax::ptr::P;

use aster;

use field::field_alias;

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

    let generics = match item.node {
        ast::ItemStruct(_, ref generics) => generics,
        ast::ItemEnum(_, ref generics) => generics,
        _ => cx.bug("expected ItemStruct or ItemEnum in derive(Deserialize)")
    };

    let trait_generics = builder.from_generics(generics.clone())
        .add_ty_param_bound(
            builder.path().global().ids(&["serde", "de", "Deserialize"]).build()
        )
        .build();

    let type_generics = builder.from_generics(trait_generics.clone())
        .strip_bounds()
        .build();

    let visitor_ty = builder.ty().path()
        .segment("__Visitor").with_generics(trait_generics.clone()).build()
        .build();

    // Build `__Visitor<A, B, ...>(PhantomData<A>, PhantomData<B>, ...)`
    let (visitor_item, visitor_expr) = deserialize_visitor(
        &builder,
        &trait_generics
    );

    let value_ty = builder.ty().path()
        .segment(substr.type_ident).with_generics(trait_generics.clone()).build()
        .build();

    match *substr.fields {
        StaticStruct(ref struct_def, ref fields) => {
            deserialize_struct(
                cx,
                span,
                &builder,
                substr.type_ident,
                substr.type_ident,
                builder.path().id(substr.type_ident).build(),
                fields,
                state,
                struct_def,
                &trait_generics,
                visitor_item,
                visitor_ty,
                visitor_expr,
                value_ty,
            )
        }
        StaticEnum(ref enum_def, ref fields) => {
            deserialize_enum(
                cx,
                &builder,
                substr.type_ident,
                &fields,
                state,
                enum_def,
                &trait_generics,
                &type_generics,
                visitor_item,
                visitor_ty,
                visitor_expr,
                value_ty,
            )
        }
        _ => cx.bug("expected StaticEnum or StaticStruct in derive(Deserialize)")
    }
}

// Build `__Visitor<A, B, ...>(PhantomData<A>, PhantomData<B>, ...)`
fn deserialize_visitor(
    builder: &aster::AstBuilder,
    trait_generics: &ast::Generics,
) -> (P<ast::Item>, P<ast::Expr>) {
    if trait_generics.ty_params.is_empty() {
        (
            builder.item().tuple_struct("__Visitor")
                .build(),
            builder.expr().id("__Visitor"),
        )
    } else {
        (
            builder.item().tuple_struct("__Visitor")
                .generics().with(trait_generics.clone()).build()
                .with_tys(
                    trait_generics.ty_params.iter().map(|ty_param| {
                        builder.ty().phantom_data().id(ty_param.ident)
                    })
                )
                .build(),
            builder.expr().call().id("__Visitor")
                .with_args(
                    trait_generics.ty_params.iter().map(|_| {
                        builder.expr().phantom_data()
                    })
                )
                .build(),
        )
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
    trait_generics: &ast::Generics,
    visitor_item: P<ast::Item>,
    visitor_ty: P<ast::Ty>,
    visitor_expr: P<ast::Expr>,
    value_ty: P<ast::Ty>,
) -> P<ast::Expr> {
    match *fields {
        Unnamed(ref fields) if fields.is_empty() => {
            deserialize_unit_struct(
                cx,
                builder,
                type_ident,
                struct_ident,
                struct_path,
                state,
            )
        }
        Unnamed(ref fields) => {
            deserialize_tuple_struct(
                cx,
                builder,
                struct_ident,
                struct_path,
                &fields,
                state,
                trait_generics,
                visitor_item,
                visitor_ty,
                visitor_expr,
                value_ty,
            )
        }
        Named(ref fields) => {
            deserialize_struct_named_fields(
                cx,
                span,
                builder,
                struct_ident,
                struct_path,
                &fields,
                state,
                struct_def,
                trait_generics,
                visitor_item,
                visitor_ty,
                visitor_expr,
                value_ty,
            )
        }
    }
}

fn deserialize_unit_struct(
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

fn deserialize_tuple_struct(
    cx: &ExtCtxt,
    builder: &aster::AstBuilder,
    struct_ident: Ident,
    struct_path: ast::Path,
    fields: &[Span],
    state: P<ast::Expr>,
    trait_generics: &ast::Generics,
    visitor_item: P<ast::Item>,
    visitor_ty: P<ast::Ty>,
    visitor_expr: P<ast::Expr>,
    value_ty: P<ast::Ty>,
) -> P<ast::Expr> {
    let where_clause = &trait_generics.where_clause;

    let field_names: Vec<ast::Ident> = (0 .. fields.len())
        .map(|i| builder.id(&format!("__field{}", i)))
        .collect();

    let visit_seq_expr = deserialize_seq(
        cx,
        builder,
        struct_path,
        &field_names,
    );

    let struct_name = builder.expr().str(struct_ident);

    quote_expr!(cx, {
        $visitor_item

        impl $trait_generics ::serde::de::Visitor for $visitor_ty $where_clause {
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

fn deserialize_seq(
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
    struct_ident: Ident,
    struct_path: ast::Path,
    fields: &[(Ident, Span)],
    state: P<ast::Expr>,
    struct_def: &StructDef,
    trait_generics: &ast::Generics,
    visitor_item: P<ast::Item>,
    visitor_ty: P<ast::Ty>,
    visitor_expr: P<ast::Expr>,
    value_ty: P<ast::Ty>,
) -> P<ast::Expr> {
    let where_clause = &trait_generics.where_clause;

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

    let struct_name = builder.expr().str(struct_ident);

    quote_expr!(cx, {
        $field_devisitor

        $visitor_item

        impl $trait_generics ::serde::de::Visitor for $visitor_ty $where_clause {
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
    let aliases: Vec<Option<&ast::Lit>> = struct_def.fields.iter()
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
    trait_generics: &ast::Generics,
    type_generics: &ast::Generics,
    visitor_item: P<ast::Item>,
    visitor_ty: P<ast::Ty>,
    visitor_expr: P<ast::Expr>,
    value_ty: P<ast::Ty>,
) -> P<ast::Expr> {
    let where_clause = &trait_generics.where_clause;

    let type_name = builder.expr().str(type_ident);

    // Match arms to extract a variant from a string
    let variant_arms: Vec<ast::Arm> = fields.iter()
        .zip(enum_def.variants.iter())
        .map(|(&(name, span, ref fields), variant)| {
            let value = deserialize_enum_variant(
                cx,
                span,
                builder,
                type_ident,
                name,
                fields,
                cx.expr_ident(span, cx.ident_of("visitor")),
                variant,
                visitor_item.clone(),
                visitor_ty.clone(),
                visitor_expr.clone(),
                &value_ty,
                &trait_generics,
            );

            let s = builder.expr().str(name);
            quote_arm!(cx, $s => $value,)
        })
        .collect();

    quote_expr!(cx, {
        $visitor_item

        impl $trait_generics ::serde::de::Visitor for __Visitor $type_generics $where_clause {
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
    variant: &P<ast::Variant>,
    visitor_item: P<ast::Item>,
    visitor_ty: P<ast::Ty>,
    visitor_expr: P<ast::Expr>,
    value_ty: &P<ast::Ty>,
    trait_generics: &ast::Generics,
) -> P<ast::Expr> {
    let variant_path = builder.path()
        .ids(&[type_ident, variant_ident])
        .build();

    match *fields {
        Unnamed(ref fields) if fields.is_empty() => {
            quote_expr!(cx, {
                try!($state.visit_unit());
                Ok($variant_path)
            })
        }

        Unnamed(ref fields) => {
            deserialize_enum_variant_seq(
                cx,
                builder,
                &*fields,
                variant_path,
                trait_generics,
                state,
                visitor_ty,
                value_ty,
            )
        }
        Named(ref fields) => {
            deserialize_enum_variant_map(
                cx,
                span,
                builder,
                &*fields,
                variant_path,
                trait_generics,
                state,
                visitor_item,
                visitor_ty,
                visitor_expr,
                value_ty,
                variant,
            )
        }
    }
}

fn deserialize_enum_variant_seq(
    cx: &ExtCtxt,
    builder: &aster::AstBuilder,
    fields: &[Span],
    variant_path: ast::Path,
    trait_generics: &ast::Generics,
    state: P<ast::Expr>,
    visitor_ty: P<ast::Ty>,
    value_ty: &P<ast::Ty>,
) -> P<ast::Expr> {
    let where_clause = &trait_generics.where_clause;

    // Create the field names for the fields.
    let field_names: Vec<ast::Ident> = (0 .. fields.len())
        .map(|i| token::str_to_ident(&format!("__field{}", i)))
        .collect();

    let (visitor_item, visitor_expr) = deserialize_visitor(
        builder,
        trait_generics,
    );

    let visit_seq_expr = deserialize_seq(
        cx,
        builder,
        variant_path,
        &field_names,
    );

    quote_expr!(cx, {
        $visitor_item

        impl $trait_generics ::serde::de::EnumSeqVisitor
        for $visitor_ty
        $where_clause {
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

fn deserialize_enum_variant_map(
    cx: &ExtCtxt,
    span: Span,
    builder: &aster::AstBuilder,
    fields: &[(Ident, Span)],
    variant_path: ast::Path,
    trait_generics: &ast::Generics,
    state: P<ast::Expr>,
    visitor_item: P<ast::Item>,
    visitor_ty: P<ast::Ty>,
    visitor_expr: P<ast::Expr>,
    value_ty: &P<ast::Ty>,
    variant: &P<ast::Variant>,
) -> P<ast::Expr> {
    let where_clause = &trait_generics.where_clause;

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
        match variant.node.kind {
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
        match variant.node.kind {
            ast::VariantKind::StructVariantKind(ref sd) => &*sd,
            _ => panic!("Mismatched enum types")
        },
    );

    quote_expr!(cx, {
        $field_devisitor

        $visitor_item

        impl $trait_generics ::serde::de::EnumMapVisitor
        for $visitor_ty
        $where_clause {
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
