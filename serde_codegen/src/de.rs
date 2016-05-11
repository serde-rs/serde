use aster;

use syntax::ast::{
    self,
    EnumDef,
    Ident,
    Item,
    MetaItem,
};
use syntax::codemap::Span;
use syntax::ext::base::{Annotatable, ExtCtxt};
use syntax::ext::build::AstBuilder;
use syntax::parse::token::InternedString;
use syntax::ptr::P;

use attr;
use bound;
use error::Error;

pub fn expand_derive_deserialize(
    cx: &mut ExtCtxt,
    span: Span,
    meta_item: &MetaItem,
    annotatable: &Annotatable,
    push: &mut FnMut(Annotatable)
) {
    let item = match *annotatable {
        Annotatable::Item(ref item) => item,
        _ => {
            cx.span_err(
                meta_item.span,
                "`#[derive(Deserialize)]` may only be applied to structs and enums");
            return;
        }
    };

    let builder = aster::AstBuilder::new().span(span);

    let generics = match item.node {
        ast::ItemKind::Struct(_, ref generics) => generics,
        ast::ItemKind::Enum(_, ref generics) => generics,
        _ => {
            cx.span_err(
                meta_item.span,
                "`#[derive(Deserialize)]` may only be applied to structs and enums");
            return;
        }
    };

    let impl_generics = build_impl_generics(cx, &builder, item, generics);

    let ty = builder.ty().path()
        .segment(item.ident).with_generics(impl_generics.clone()).build()
        .build();

    let body = match deserialize_body(cx, &builder, &item, &impl_generics, ty.clone()) {
        Ok(body) => body,
        Err(Error) => {
            // An error occured, but it should have been reported already.
            return;
        }
    };

    let where_clause = &impl_generics.where_clause;

    let dummy_const = builder.id(format!("_IMPL_DESERIALIZE_FOR_{}", item.ident));

    let impl_item = quote_item!(cx,
        #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
        const $dummy_const: () = {
            extern crate serde as _serde;
            #[automatically_derived]
            impl $impl_generics _serde::de::Deserialize for $ty $where_clause {
                fn deserialize<__D>(deserializer: &mut __D) -> ::std::result::Result<$ty, __D::Error>
                    where __D: _serde::de::Deserializer,
                {
                    $body
                }
            }
        };
    ).unwrap();

    push(Annotatable::Item(impl_item))
}

// All the generics in the input, plus a bound `T: Deserialize` for each generic
// field type that will be deserialized by us, plus a bound `T: Default` for
// each generic field type that will be set to a default value.
fn build_impl_generics(
    cx: &ExtCtxt,
    builder: &aster::AstBuilder,
    item: &Item,
    generics: &ast::Generics,
) -> ast::Generics {
    let generics = bound::without_defaults(generics);
    let generics = bound::with_bound(cx, builder, item, &generics,
        &deserialized_by_us,
        &builder.path().ids(&["_serde", "de", "Deserialize"]).build());
    let generics = bound::with_bound(cx, builder, item, &generics,
        &requires_default,
        &builder.path().global().ids(&["std", "default", "Default"]).build());
    generics
}

// Fields with a `skip_deserializing` or `deserialize_with` attribute are not
// deserialized by us. All other fields may need a `T: Deserialize` bound where
// T is the type of the field.
fn deserialized_by_us(field: &ast::StructField) -> bool {
    for meta_items in field.attrs.iter().filter_map(attr::get_serde_meta_items) {
        for meta_item in meta_items {
            match meta_item.node {
                ast::MetaItemKind::Word(ref name) if name == &"skip_deserializing" => {
                    return false
                }
                ast::MetaItemKind::NameValue(ref name, _) if name == &"deserialize_with" => {
                    return false
                }
                _ => {}
            }
        }
    }
    true
}

// Fields with a `default` attribute (not `default=...`), and fields with a
// `skip_deserializing` attribute that do not also have `default=...`.
fn requires_default(field: &ast::StructField) -> bool {
    let mut has_skip_deserializing = false;
    for meta_items in field.attrs.iter().filter_map(attr::get_serde_meta_items) {
        for meta_item in meta_items {
            match meta_item.node {
                ast::MetaItemKind::Word(ref name) if name == &"default" => {
                    return true
                }
                ast::MetaItemKind::NameValue(ref name, _) if name == &"default" => {
                    return false
                }
                ast::MetaItemKind::Word(ref name) if name == &"skip_deserializing" => {
                    has_skip_deserializing = true
                }
                _ => {}
            }
        }
    }
    has_skip_deserializing
}

fn deserialize_body(
    cx: &ExtCtxt,
    builder: &aster::AstBuilder,
    item: &Item,
    impl_generics: &ast::Generics,
    ty: P<ast::Ty>,
) -> Result<P<ast::Expr>, Error> {
    let container_attrs = try!(attr::ContainerAttrs::from_item(cx, item));

    match item.node {
        ast::ItemKind::Struct(ref variant_data, _) => {
            deserialize_item_struct(
                cx,
                builder,
                item,
                impl_generics,
                ty,
                item.span,
                variant_data,
                &container_attrs,
            )
        }
        ast::ItemKind::Enum(ref enum_def, _) => {
            deserialize_item_enum(
                cx,
                builder,
                item.ident,
                impl_generics,
                ty,
                enum_def,
                &container_attrs,
            )
        }
        _ => {
            cx.span_bug(item.span,
                        "expected ItemStruct or ItemEnum in #[derive(Deserialize)]")
        }
    }
}

fn deserialize_item_struct(
    cx: &ExtCtxt,
    builder: &aster::AstBuilder,
    item: &Item,
    impl_generics: &ast::Generics,
    ty: P<ast::Ty>,
    span: Span,
    variant_data: &ast::VariantData,
    container_attrs: &attr::ContainerAttrs,
) -> Result<P<ast::Expr>, Error> {
    match *variant_data {
        ast::VariantData::Unit(_) => {
            deserialize_unit_struct(
                cx,
                item.ident,
                container_attrs,
            )
        }
        ast::VariantData::Tuple(ref fields, _) if fields.len() == 1 => {
            deserialize_newtype_struct(
                cx,
                &builder,
                item.ident,
                impl_generics,
                ty,
                container_attrs,
            )
        }
        ast::VariantData::Tuple(ref fields, _) => {
            if fields.iter().any(|field| field.ident.is_some()) {
                cx.span_bug(span, "tuple struct has named fields")
            }

            deserialize_tuple_struct(
                cx,
                &builder,
                item.ident,
                impl_generics,
                ty,
                fields.len(),
                container_attrs,
            )
        }
        ast::VariantData::Struct(ref fields, _) => {
            if fields.iter().any(|field| field.ident.is_none()) {
                cx.span_bug(span, "struct has unnamed fields")
            }

            deserialize_struct(
                cx,
                &builder,
                item.ident,
                impl_generics,
                ty,
                fields,
                container_attrs,
            )
        }
    }
}


// Build `__Visitor<A, B, ...>(PhantomData<A>, PhantomData<B>, ...)`
fn deserialize_visitor(
    builder: &aster::AstBuilder,
    trait_generics: &ast::Generics,
    forward_ty_params: Vec<ast::TyParam>,
    forward_tys: Vec<P<ast::Ty>>
) -> Result<(P<ast::Item>, P<ast::Ty>, P<ast::Expr>, ast::Generics), Error> {
    if trait_generics.ty_params.is_empty() && forward_tys.is_empty() {
        Ok((
            builder.item().tuple_struct("__Visitor").build(),
            builder.ty().id("__Visitor"),
            builder.expr().id("__Visitor"),
            trait_generics.clone(),
        ))
    } else {
        let placeholders : Vec<_> = trait_generics.ty_params.iter()
            .map(|t| builder.ty().id(t.ident))
            .collect();
        let mut trait_generics = trait_generics.clone();
        let mut ty_params = forward_ty_params.clone();
        ty_params.extend(trait_generics.ty_params.into_vec());
        trait_generics.ty_params = P::from_vec(ty_params);

        Ok((
            builder.item().tuple_struct("__Visitor")
                .generics().with(trait_generics.clone()).build()
                .with_tys({
                    let lifetimes = trait_generics.lifetimes.iter()
                        .map(|lifetime_def| {
                            builder.ty()
                                .phantom_data()
                                .ref_().lifetime(lifetime_def.lifetime.name)
                                .ty()
                                .unit()
                        });

                    let ty_params = trait_generics.ty_params.iter()
                        .map(|ty_param| {
                            builder.ty()
                                .phantom_data()
                                .id(ty_param.ident)
                        });

                    lifetimes.chain(ty_params)
                })
                .build(),
            builder.ty().path()
                .segment("__Visitor").with_generics(trait_generics.clone()).build()
                .build(),
            builder.expr().call()
                .path().segment("__Visitor")
                .with_tys(forward_tys)
                .with_tys(placeholders)
                .build().build()
                .with_args({
                    let len = trait_generics.lifetimes.len() + trait_generics.ty_params.len();

                    (0 .. len).map(|_| builder.expr().phantom_data())
                })
                .build(),
            trait_generics,
        ))
    }
}

fn deserializer_ty_param(builder: &aster::AstBuilder) -> ast::TyParam {
    builder.ty_param("__D")
        .trait_bound(builder.path()
                     .segment("_serde").build()
                     .segment("de").build()
                     .id("Deserializer")
                     .build())
        .build()
        .build()
}

fn deserializer_ty_arg(builder: &aster::AstBuilder) -> P<ast::Ty>{
    builder.ty().id("__D")
}

fn deserialize_unit_struct(
    cx: &ExtCtxt,
    type_ident: Ident,
    container_attrs: &attr::ContainerAttrs,
) -> Result<P<ast::Expr>, Error> {
    let type_name = container_attrs.name().deserialize_name_expr();

    Ok(quote_expr!(cx, {
        struct __Visitor;

        impl _serde::de::Visitor for __Visitor {
            type Value = $type_ident;

            #[inline]
            fn visit_unit<__E>(&mut self) -> ::std::result::Result<$type_ident, __E>
                where __E: _serde::de::Error,
            {
                Ok($type_ident)
            }

            #[inline]
            fn visit_seq<__V>(&mut self, mut visitor: __V) -> ::std::result::Result<$type_ident, __V::Error>
                where __V: _serde::de::SeqVisitor,
            {
                try!(visitor.end());
                self.visit_unit()
            }
        }

        deserializer.deserialize_unit_struct($type_name, __Visitor)
    }))
}

fn deserialize_newtype_struct(
    cx: &ExtCtxt,
    builder: &aster::AstBuilder,
    type_ident: Ident,
    impl_generics: &ast::Generics,
    ty: P<ast::Ty>,
    container_attrs: &attr::ContainerAttrs,
) -> Result<P<ast::Expr>, Error> {
    let where_clause = &impl_generics.where_clause;

    let (visitor_item, visitor_ty, visitor_expr, visitor_generics) = try!(deserialize_visitor(
        builder,
        impl_generics,
        vec![deserializer_ty_param(builder)],
        vec![deserializer_ty_arg(builder)],
    ));

    let visit_seq_expr = deserialize_seq(
        cx,
        builder,
        builder.path().id(type_ident).build(),
        1,
    );

    let type_name = container_attrs.name().deserialize_name_expr();

    Ok(quote_expr!(cx, {
        $visitor_item

        impl $visitor_generics _serde::de::Visitor for $visitor_ty $where_clause {
            type Value = $ty;

            #[inline]
            fn visit_newtype_struct<__E>(&mut self, deserializer: &mut __E) -> ::std::result::Result<Self::Value, __E::Error>
                where __E: _serde::de::Deserializer,
            {
                let value = try!(_serde::de::Deserialize::deserialize(deserializer));
                Ok($type_ident(value))
            }

            #[inline]
            fn visit_seq<__V>(&mut self, mut visitor: __V) -> ::std::result::Result<$ty, __V::Error>
                where __V: _serde::de::SeqVisitor,
            {
                $visit_seq_expr
            }
        }

        deserializer.deserialize_newtype_struct($type_name, $visitor_expr)
    }))
}

fn deserialize_tuple_struct(
    cx: &ExtCtxt,
    builder: &aster::AstBuilder,
    type_ident: Ident,
    impl_generics: &ast::Generics,
    ty: P<ast::Ty>,
    fields: usize,
    container_attrs: &attr::ContainerAttrs,
) -> Result<P<ast::Expr>, Error> {
    let where_clause = &impl_generics.where_clause;

    let (visitor_item, visitor_ty, visitor_expr, visitor_generics) = try!(deserialize_visitor(
        builder,
        impl_generics,
        vec![deserializer_ty_param(builder)],
        vec![deserializer_ty_arg(builder)],
    ));

    let visit_seq_expr = deserialize_seq(
        cx,
        builder,
        builder.path().id(type_ident).build(),
        fields,
    );

    let type_name = container_attrs.name().deserialize_name_expr();

    Ok(quote_expr!(cx, {
        $visitor_item

        impl $visitor_generics _serde::de::Visitor for $visitor_ty $where_clause {
            type Value = $ty;

            #[inline]
            fn visit_seq<__V>(&mut self, mut visitor: __V) -> ::std::result::Result<$ty, __V::Error>
                where __V: _serde::de::SeqVisitor,
            {
                $visit_seq_expr
            }
        }

        deserializer.deserialize_tuple_struct($type_name, $fields, $visitor_expr)
    }))
}

fn deserialize_seq(
    cx: &ExtCtxt,
    builder: &aster::AstBuilder,
    struct_path: ast::Path,
    fields: usize,
) -> P<ast::Expr> {
    let let_values: Vec<ast::Stmt> = (0 .. fields)
        .map(|i| {
            let name = builder.id(format!("__field{}", i));
            quote_stmt!(cx,
                let $name = match try!(visitor.visit()) {
                    Some(value) => { value },
                    None => {
                        return Err(_serde::de::Error::end_of_stream());
                    }
                };
            ).unwrap()
        })
        .collect();

    let result = builder.expr().call()
        .build_path(struct_path)
        .with_args((0 .. fields).map(|i| builder.expr().id(format!("__field{}", i))))
        .build();

    quote_expr!(cx, {
        $let_values

        try!(visitor.end());

        Ok($result)
    })
}

fn deserialize_struct_as_seq(
    cx: &ExtCtxt,
    builder: &aster::AstBuilder,
    type_ident: Ident,
    struct_path: ast::Path,
    impl_generics: &ast::Generics,
    fields: &[(&ast::StructField, attr::FieldAttrs)],
) -> Result<P<ast::Expr>, Error> {
    let let_values: Vec<_> = fields.iter()
        .enumerate()
        .map(|(i, &(field, ref attrs))| {
            let name = builder.id(format!("__field{}", i));
            if attrs.skip_deserializing_field() {
                let default = expr_is_missing(cx, attrs);
                quote_stmt!(cx,
                    let $name = $default;
                ).unwrap()
            } else {
                let visit = match attrs.deserialize_with() {
                    None => {
                        let field_ty = &field.ty;
                        quote_expr!(cx, try!(visitor.visit::<$field_ty>()))
                    }
                    Some(path) => {
                        let (wrapper, wrapper_impl, wrapper_ty) = wrap_deserialize_with(
                            cx, builder, type_ident, impl_generics, &field.ty, path);
                        quote_expr!(cx, {
                            $wrapper
                            $wrapper_impl
                            try!(visitor.visit::<$wrapper_ty>()).map(|wrap| wrap.value)
                        })
                    }
                };
                quote_stmt!(cx,
                    let $name = match $visit {
                        Some(value) => { value },
                        None => {
                            return Err(_serde::de::Error::end_of_stream());
                        }
                    };
                ).unwrap()
            }
        })
        .collect();

    let result = builder.expr().struct_path(struct_path)
        .with_id_exprs(
            fields.iter()
                .enumerate()
                .map(|(i, &(field, _))| {
                    (
                        match field.ident {
                            Some(name) => name.clone(),
                            None => {
                                cx.span_bug(field.span, "struct contains unnamed fields")
                            }
                        },
                        builder.expr().id(format!("__field{}", i)),
                    )
                })
        )
        .build();

    Ok(quote_expr!(cx, {
        $let_values

        try!(visitor.end());

        Ok($result)
    }))
}

fn deserialize_struct(
    cx: &ExtCtxt,
    builder: &aster::AstBuilder,
    type_ident: Ident,
    impl_generics: &ast::Generics,
    ty: P<ast::Ty>,
    fields: &[ast::StructField],
    container_attrs: &attr::ContainerAttrs,
) -> Result<P<ast::Expr>, Error> {
    let where_clause = &impl_generics.where_clause;

    let (visitor_item, visitor_ty, visitor_expr, visitor_generics) = try!(deserialize_visitor(
        builder,
        &impl_generics,
        vec![deserializer_ty_param(builder)],
        vec![deserializer_ty_arg(builder)],
    ));

    let type_path = builder.path().id(type_ident).build();

    let fields_with_attrs = try!(fields_with_attrs(cx, impl_generics, &ty, fields, false));

    let visit_seq_expr = try!(deserialize_struct_as_seq(
        cx,
        builder,
        type_ident,
        type_path.clone(),
        impl_generics,
        &fields_with_attrs,
    ));

    let (field_visitor, fields_stmt, visit_map_expr) = try!(deserialize_struct_visitor(
        cx,
        builder,
        type_ident,
        type_path.clone(),
        impl_generics,
        &fields_with_attrs,
        container_attrs,
    ));

    let type_name = container_attrs.name().deserialize_name_expr();

    Ok(quote_expr!(cx, {
        $field_visitor

        $visitor_item

        impl $visitor_generics _serde::de::Visitor for $visitor_ty $where_clause {
            type Value = $ty;

            #[inline]
            fn visit_seq<__V>(&mut self, mut visitor: __V) -> ::std::result::Result<$ty, __V::Error>
                where __V: _serde::de::SeqVisitor,
            {
                $visit_seq_expr
            }

            #[inline]
            fn visit_map<__V>(&mut self, mut visitor: __V) -> ::std::result::Result<$ty, __V::Error>
                where __V: _serde::de::MapVisitor,
            {
                $visit_map_expr
            }
        }

        $fields_stmt

        deserializer.deserialize_struct($type_name, FIELDS, $visitor_expr)
    }))
}

fn deserialize_item_enum(
    cx: &ExtCtxt,
    builder: &aster::AstBuilder,
    type_ident: Ident,
    impl_generics: &ast::Generics,
    ty: P<ast::Ty>,
    enum_def: &EnumDef,
    container_attrs: &attr::ContainerAttrs
) -> Result<P<ast::Expr>, Error> {
    let where_clause = &impl_generics.where_clause;

    let type_name = container_attrs.name().deserialize_name_expr();

    let variant_visitor = deserialize_field_visitor(
        cx,
        builder,
        try!(
            enum_def.variants.iter()
                .map(|variant| {
                    let attrs = try!(attr::VariantAttrs::from_variant(cx, variant));
                    Ok(attrs.name().deserialize_name())
                })
                .collect()
        ),
        container_attrs,
        true,
    );

    let variants_expr = builder.expr().ref_().slice()
        .with_exprs(
            enum_def.variants.iter()
                .map(|variant| {
                    builder.expr().str(variant.node.name)
                })
        )
        .build();

    let variants_stmt = quote_stmt!(cx,
        const VARIANTS: &'static [&'static str] = $variants_expr;
    ).unwrap();

    let ignored_arm = if container_attrs.deny_unknown_fields() {
        None
    } else {
        Some(quote_arm!(cx, __Field::__ignore => { Err(_serde::de::Error::end_of_stream()) }))
    };

    // Match arms to extract a variant from a string
    let mut variant_arms = vec![];
    for (i, variant) in enum_def.variants.iter().enumerate() {
        let variant_name = builder.pat().path()
            .id("__Field").id(format!("__field{}", i))
            .build();

        let expr = try!(deserialize_variant(
            cx,
            builder,
            type_ident,
            impl_generics,
            ty.clone(),
            variant,
            container_attrs,
        ));

        let arm = quote_arm!(cx, $variant_name => { $expr });
        variant_arms.push(arm);
    }
    variant_arms.extend(ignored_arm.into_iter());

    let (visitor_item, visitor_ty, visitor_expr, visitor_generics) = try!(deserialize_visitor(
        builder,
        impl_generics,
        vec![deserializer_ty_param(builder)],
        vec![deserializer_ty_arg(builder)],
    ));

    Ok(quote_expr!(cx, {
        $variant_visitor

        $visitor_item

        impl $visitor_generics _serde::de::EnumVisitor for $visitor_ty $where_clause {
            type Value = $ty;

            fn visit<__V>(&mut self, mut visitor: __V) -> ::std::result::Result<$ty, __V::Error>
                where __V: _serde::de::VariantVisitor,
            {
                match try!(visitor.visit_variant()) {
                    $variant_arms
                }
            }
        }

        $variants_stmt

        deserializer.deserialize_enum($type_name, VARIANTS, $visitor_expr)
    }))
}

fn deserialize_variant(
    cx: &ExtCtxt,
    builder: &aster::AstBuilder,
    type_ident: Ident,
    generics: &ast::Generics,
    ty: P<ast::Ty>,
    variant: &ast::Variant,
    container_attrs: &attr::ContainerAttrs,
) -> Result<P<ast::Expr>, Error> {
    let variant_ident = variant.node.name;

    match variant.node.data {
        ast::VariantData::Unit(_) => {
            Ok(quote_expr!(cx, {
                try!(visitor.visit_unit());
                Ok($type_ident::$variant_ident)
            }))
        }
        ast::VariantData::Tuple(ref args, _) if args.len() == 1 => {
            Ok(quote_expr!(cx, {
                let val = try!(visitor.visit_newtype());
                Ok($type_ident::$variant_ident(val))
            }))
        }
        ast::VariantData::Tuple(ref fields, _) => {
            deserialize_tuple_variant(
                cx,
                builder,
                type_ident,
                variant_ident,
                generics,
                ty,
                fields.len(),
            )
        }
        ast::VariantData::Struct(ref fields, _) => {
            deserialize_struct_variant(
                cx,
                builder,
                type_ident,
                variant_ident,
                generics,
                ty,
                fields,
                container_attrs,
            )
        }
    }
}

fn deserialize_tuple_variant(
    cx: &ExtCtxt,
    builder: &aster::AstBuilder,
    type_ident: ast::Ident,
    variant_ident: ast::Ident,
    generics: &ast::Generics,
    ty: P<ast::Ty>,
    fields: usize,
) -> Result<P<ast::Expr>, Error> {
    let where_clause = &generics.where_clause;

    let (visitor_item, visitor_ty, visitor_expr, visitor_generics) = try!(deserialize_visitor(
        builder,
        generics,
        vec![deserializer_ty_param(builder)],
        vec![deserializer_ty_arg(builder)],
    ));

    let visit_seq_expr = deserialize_seq(
        cx,
        builder,
        builder.path().id(type_ident).id(variant_ident).build(),
        fields,
    );

    Ok(quote_expr!(cx, {
        $visitor_item

        impl $visitor_generics _serde::de::Visitor for $visitor_ty $where_clause {
            type Value = $ty;

            fn visit_seq<__V>(&mut self, mut visitor: __V) -> ::std::result::Result<$ty, __V::Error>
                where __V: _serde::de::SeqVisitor,
            {
                $visit_seq_expr
            }
        }

        visitor.visit_tuple($fields, $visitor_expr)
    }))
}

fn deserialize_struct_variant(
    cx: &ExtCtxt,
    builder: &aster::AstBuilder,
    type_ident: ast::Ident,
    variant_ident: ast::Ident,
    generics: &ast::Generics,
    ty: P<ast::Ty>,
    fields: &[ast::StructField],
    container_attrs: &attr::ContainerAttrs,
) -> Result<P<ast::Expr>, Error> {
    let where_clause = &generics.where_clause;

    let type_path = builder.path()
        .id(type_ident)
        .id(variant_ident)
        .build();

    let fields_with_attrs = try!(fields_with_attrs(cx, generics, &ty, fields, true));

    let visit_seq_expr = try!(deserialize_struct_as_seq(
        cx,
        builder,
        type_ident,
        type_path.clone(),
        generics,
        &fields_with_attrs,
    ));

    let (field_visitor, fields_stmt, field_expr) = try!(deserialize_struct_visitor(
        cx,
        builder,
        type_ident,
        type_path,
        generics,
        &fields_with_attrs,
        container_attrs,
    ));

    let (visitor_item, visitor_ty, visitor_expr, visitor_generics) = try!(deserialize_visitor(
        builder,
        generics,
        vec![deserializer_ty_param(builder)],
        vec![deserializer_ty_arg(builder)],
    ));

    Ok(quote_expr!(cx, {
        $field_visitor

        $visitor_item

        impl $visitor_generics _serde::de::Visitor for $visitor_ty $where_clause {
            type Value = $ty;

            #[inline]
            fn visit_seq<__V>(&mut self, mut visitor: __V) -> ::std::result::Result<$ty, __V::Error>
                where __V: _serde::de::SeqVisitor,
            {
                $visit_seq_expr
            }

            #[inline]
            fn visit_map<__V>(&mut self, mut visitor: __V) -> ::std::result::Result<$ty, __V::Error>
                where __V: _serde::de::MapVisitor,
            {
                $field_expr
            }
        }

        $fields_stmt

        visitor.visit_struct(FIELDS, $visitor_expr)
    }))
}

fn deserialize_field_visitor(
    cx: &ExtCtxt,
    builder: &aster::AstBuilder,
    field_names: Vec<InternedString>,
    container_attrs: &attr::ContainerAttrs,
    is_variant: bool,
) -> Vec<P<ast::Item>> {
    // Create the field names for the fields.
    let field_idents: Vec<_> = (0 .. field_names.len())
        .map(|i| builder.id(format!("__field{}", i)))
        .collect();

    let ignore_variant = if container_attrs.deny_unknown_fields() {
        None
    } else {
        let skip_ident = builder.id("__ignore");
        Some(builder.variant(skip_ident).unit())
    };

    let field_enum = builder.item()
        .attr().allow(&["non_camel_case_types"])
        .enum_("__Field")
        .with_variants(
            field_idents.iter().map(|field_ident| {
                builder.variant(field_ident).unit()
            })
        )
        .with_variants(ignore_variant.into_iter())
        .build();

    let index_field_arms: Vec<_> = field_idents.iter()
        .enumerate()
        .map(|(field_index, field_ident)| {
            quote_arm!(cx, $field_index => { Ok(__Field::$field_ident) })
        })
        .collect();

    let (index_error_msg, unknown_ident) = if is_variant {
        (builder.expr().str("expected a variant"), builder.id("unknown_variant"))
    } else {
        (builder.expr().str("expected a field"), builder.id("unknown_field"))
    };

    let fallthrough_index_arm_expr = if !is_variant && !container_attrs.deny_unknown_fields() {
        quote_expr!(cx, Ok(__Field::__ignore))
    } else {
        quote_expr!(cx, {
            Err(_serde::de::Error::invalid_value($index_error_msg))
        })
    };

    let index_body = quote_expr!(cx,
        match value {
            $index_field_arms
            _ => $fallthrough_index_arm_expr
        }
    );

    // Convert the field names into byte strings.
    let str_field_names: Vec<_> = field_names.iter()
        .map(|name| builder.expr().lit().str(&name))
        .collect();

    // Match arms to extract a field from a string
    let str_field_arms: Vec<_> = field_idents.iter().zip(str_field_names.iter())
        .map(|(field_ident, field_name)| {
            quote_arm!(cx, $field_name => { Ok(__Field::$field_ident) })
        })
        .collect();

    let fallthrough_str_arm_expr = if !is_variant && !container_attrs.deny_unknown_fields() {
        quote_expr!(cx, Ok(__Field::__ignore))
    } else {
        quote_expr!(cx, Err(_serde::de::Error::$unknown_ident(value)))
    };

    let str_body = quote_expr!(cx,
        match value {
            $str_field_arms
            _ => $fallthrough_str_arm_expr
        }
    );

    // Convert the field names into byte strings.
    let bytes_field_names: Vec<_> = field_names.iter()
        .map(|name| {
            let name: &str = name;
            builder.expr().lit().byte_str(name)
        })
        .collect();

    // Match arms to extract a field from a string
    let bytes_field_arms: Vec<_> = field_idents.iter().zip(bytes_field_names.iter())
        .map(|(field_ident, field_name)| {
            quote_arm!(cx, $field_name => { Ok(__Field::$field_ident) })
        })
        .collect();

    let fallthrough_bytes_arm_expr = if !is_variant && !container_attrs.deny_unknown_fields() {
        quote_expr!(cx, Ok(__Field::__ignore))
    } else {
        quote_expr!(cx, {
            let value = ::std::string::String::from_utf8_lossy(value);
            Err(_serde::de::Error::$unknown_ident(&value))
        })
    };

    let bytes_body = quote_expr!(cx,
        match value {
            $bytes_field_arms
            _ => $fallthrough_bytes_arm_expr
        }
    );

    let impl_item = quote_item!(cx,
        impl _serde::de::Deserialize for __Field {
            #[inline]
            fn deserialize<__D>(deserializer: &mut __D) -> ::std::result::Result<__Field, __D::Error>
                where __D: _serde::de::Deserializer,
            {
                struct __FieldVisitor<__D> {
                    phantom: ::std::marker::PhantomData<__D>
                }

                impl<__D> _serde::de::Visitor for __FieldVisitor<__D>
                    where __D: _serde::de::Deserializer
                {
                    type Value = __Field;

                    fn visit_usize<__E>(&mut self, value: usize) -> ::std::result::Result<__Field, __E>
                        where __E: _serde::de::Error,
                    {
                        $index_body
                    }

                    fn visit_str<__E>(&mut self, value: &str) -> ::std::result::Result<__Field, __E>
                        where __E: _serde::de::Error,
                    {
                        $str_body
                    }

                    fn visit_bytes<__E>(&mut self, value: &[u8]) -> ::std::result::Result<__Field, __E>
                        where __E: _serde::de::Error,
                    {
                        $bytes_body
                    }
                }

                deserializer.deserialize_struct_field(
                    __FieldVisitor::<__D>{
                        phantom: ::std::marker::PhantomData
                    }
                )
            }
        }
    ).unwrap();

    vec![field_enum, impl_item]
}

fn deserialize_struct_visitor(
    cx: &ExtCtxt,
    builder: &aster::AstBuilder,
    type_ident: Ident,
    struct_path: ast::Path,
    impl_generics: &ast::Generics,
    fields: &[(&ast::StructField, attr::FieldAttrs)],
    container_attrs: &attr::ContainerAttrs,
) -> Result<(Vec<P<ast::Item>>, ast::Stmt, P<ast::Expr>), Error> {
    let field_exprs = fields.iter()
        .map(|&(_, ref attrs)| attrs.name().deserialize_name())
        .collect();

    let field_visitor = deserialize_field_visitor(
        cx,
        builder,
        field_exprs,
        container_attrs,
        false,
    );

    let visit_map_expr = try!(deserialize_map(
        cx,
        builder,
        type_ident,
        struct_path,
        impl_generics,
        fields,
        container_attrs,
    ));

    let fields_expr = builder.expr().ref_().slice()
        .with_exprs(
            fields.iter()
                .map(|&(field, _)| {
                    match field.ident {
                        Some(name) => builder.expr().str(name),
                        None => {
                            cx.span_bug(field.span, "struct contains unnamed fields")
                        }
                    }
                })
        )
        .build();

    let fields_stmt = quote_stmt!(cx,
        const FIELDS: &'static [&'static str] = $fields_expr;
    ).unwrap();

    Ok((field_visitor, fields_stmt, visit_map_expr))
}

fn deserialize_map(
    cx: &ExtCtxt,
    builder: &aster::AstBuilder,
    type_ident: Ident,
    struct_path: ast::Path,
    impl_generics: &ast::Generics,
    fields: &[(&ast::StructField, attr::FieldAttrs)],
    container_attrs: &attr::ContainerAttrs,
) -> Result<P<ast::Expr>, Error> {
    // Create the field names for the fields.
    let fields_attrs_names = fields.iter()
        .enumerate()
        .map(|(i, &(ref field, ref attrs))|
             (field, attrs, builder.id(format!("__field{}", i))))
        .collect::<Vec<_>>();

    // Declare each field that will be deserialized.
    let let_values: Vec<ast::Stmt> = fields_attrs_names.iter()
        .filter(|&&(_, ref attrs, _)| !attrs.skip_deserializing_field())
        .map(|&(field, _, name)| {
            let field_ty = &field.ty;
            quote_stmt!(cx, let mut $name: Option<$field_ty> = None;).unwrap()
        })
        .collect();

    // Match arms to extract a value for a field.
    let value_arms = fields_attrs_names.iter()
        .filter(|&&(_, ref attrs, _)| !attrs.skip_deserializing_field())
        .map(|&(field, ref attrs, name)| {
            let deser_name = attrs.name().deserialize_name();
            let name_str = builder.expr().lit().str(deser_name);

            let visit = match attrs.deserialize_with() {
                None => {
                    let field_ty = &field.ty;
                    quote_expr!(cx, try!(visitor.visit_value::<$field_ty>()))
                }
                Some(path) => {
                    let (wrapper, wrapper_impl, wrapper_ty) = wrap_deserialize_with(
                        cx, builder, type_ident, impl_generics, &field.ty, path);
                    quote_expr!(cx, ({
                        $wrapper
                        $wrapper_impl
                        try!(visitor.visit_value::<$wrapper_ty>()).value
                    }))
                }
            };
            quote_arm!(cx,
                __Field::$name => {
                    if $name.is_some() {
                        return Err(<__V::Error as _serde::de::Error>::duplicate_field($name_str));
                    }
                    $name = Some($visit);
                }
            )
        })
        .collect::<Vec<_>>();

    // Match arms to ignore value for fields that have `skip_deserializing`.
    // Ignored even if `deny_unknown_fields` is set.
    let skipped_arms = fields_attrs_names.iter()
        .filter(|&&(_, ref attrs, _)| attrs.skip_deserializing_field())
        .map(|&(_, _, name)| {
            quote_arm!(cx,
                __Field::$name => {
                    try!(visitor.visit_value::<_serde::de::impls::IgnoredAny>());
                }
            )
        })
        .collect::<Vec<_>>();

    // Visit ignored values to consume them
    let ignored_arm = if container_attrs.deny_unknown_fields() {
        None
    } else {
        Some(quote_arm!(cx,
            _ => { try!(visitor.visit_value::<_serde::de::impls::IgnoredAny>()); }
        ))
    };

    let extract_values = fields_attrs_names.iter()
        .filter(|&&(_, ref attrs, _)| !attrs.skip_deserializing_field())
        .map(|&(_, ref attrs, name)| {
            let missing_expr = expr_is_missing(cx, attrs);

            Ok(quote_stmt!(cx,
                let $name = match $name {
                    Some($name) => $name,
                    None => $missing_expr
                };
            ).unwrap())
        })
        .collect::<Result<Vec<_>, _>>();

    let extract_values = try!(extract_values);

    let result = builder.expr().struct_path(struct_path)
        .with_id_exprs(
            fields_attrs_names.iter()
                .map(|&(field, attrs, name)| {
                    (
                        match field.ident {
                            Some(name) => name.clone(),
                            None => {
                                cx.span_bug(field.span, "struct contains unnamed fields")
                            }
                        },
                        if attrs.skip_deserializing_field() {
                            expr_is_missing(cx, attrs)
                        } else {
                            builder.expr().id(name)
                        }
                    )
                })
        )
        .build();

    Ok(quote_expr!(cx, {
        $let_values

        while let Some(key) = try!(visitor.visit_key::<__Field>()) {
            match key {
                $value_arms
                $skipped_arms
                $ignored_arm
            }
        }

        $extract_values

        try!(visitor.end());

        Ok($result)
    }))
}

fn fields_with_attrs<'a>(
    cx: &ExtCtxt,
    generics: &ast::Generics,
    ty: &P<ast::Ty>,
    fields: &'a [ast::StructField],
    is_enum: bool
) -> Result<Vec<(&'a ast::StructField, attr::FieldAttrs)>, Error> {
    fields.iter()
        .map(|field| {
            let attrs = try!(attr::FieldAttrs::from_field(cx, &ty, generics, field, is_enum));
            Ok((field, attrs))
        })
        .collect()
}

/// This function wraps the expression in `#[serde(deserialize_with="...")]` in
/// a trait to prevent it from accessing the internal `Deserialize` state.
fn wrap_deserialize_with(
    cx: &ExtCtxt,
    builder: &aster::AstBuilder,
    type_ident: Ident,
    impl_generics: &ast::Generics,
    field_ty: &P<ast::Ty>,
    deserialize_with: &ast::Path,
) -> (ast::Stmt, ast::Stmt, ast::Path) {
    // Quasi-quoting doesn't do a great job of expanding generics into paths,
    // so manually build it.
    let wrapper_ty = builder.path()
        .segment("__SerdeDeserializeWithStruct")
            .with_generics(impl_generics.clone())
            .build()
        .build();

    let where_clause = &impl_generics.where_clause;

    let phantom_ty = builder.path()
        .segment(type_ident)
            .with_generics(builder.from_generics(impl_generics.clone())
                .strip_ty_params()
                .build())
            .build()
        .build();

    (
        quote_stmt!(cx,
            struct __SerdeDeserializeWithStruct $impl_generics $where_clause {
                value: $field_ty,
                phantom: ::std::marker::PhantomData<$phantom_ty>,
            }
        ).unwrap(),
        quote_stmt!(cx,
            impl $impl_generics _serde::de::Deserialize for $wrapper_ty $where_clause {
                fn deserialize<__D>(__d: &mut __D) -> ::std::result::Result<Self, __D::Error>
                    where __D: _serde::de::Deserializer
                {
                    let value = try!($deserialize_with(__d));
                    Ok(__SerdeDeserializeWithStruct {
                        value: value,
                        phantom: ::std::marker::PhantomData,
                    })
                }
            }
        ).unwrap(),
        wrapper_ty,
    )
}

fn expr_is_missing(
    cx: &ExtCtxt,
    attrs: &attr::FieldAttrs,
) -> P<ast::Expr> {
    if let Some(expr) = attrs.default_expr_if_missing() {
        return expr.clone();
    }
    let name = attrs.name().deserialize_name_expr();
    match attrs.deserialize_with() {
        None => {
            quote_expr!(cx, try!(visitor.missing_field($name)))
        }
        Some(_) => {
            quote_expr!(cx, return Err(
                <__V::Error as _serde::de::Error>::missing_field($name)))
        }
    }
}
