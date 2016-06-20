use aster;

use syntax::ast::{self, Ident, MetaItem};
use syntax::codemap::Span;
use syntax::ext::base::{Annotatable, ExtCtxt};
use syntax::parse::token::InternedString;
use syntax::ptr::P;

use bound;
use error::Error;
use item::{self, attr};

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

    let item = match item::Item::from_ast(cx, item) {
        Ok(item) => item,
        Err(item::Error::UnexpectedItemKind) => {
            cx.span_err(item.span,
                "`#[derive(Deserialize)]` may only be applied to structs and enums");
            return;
        }
    };

    if check_no_str(cx, &item).is_err() {
        return;
    }

    let builder = aster::AstBuilder::new().span(span);

    let impl_item = deserialize_item(cx, &builder, &item);
    push(Annotatable::Item(impl_item))
}

fn deserialize_item(
    cx: &ExtCtxt,
    builder: &aster::AstBuilder,
    item: &item::Item,
) -> P<ast::Item> {
    let impl_generics = build_impl_generics(builder, &item);

    let ty = builder.ty().path()
        .segment(item.ident).with_generics(impl_generics.clone()).build()
        .build();

    let body = deserialize_body(cx,
                                builder,
                                &item,
                                &impl_generics,
                                ty.clone());

    let where_clause = &impl_generics.where_clause;

    let dummy_const = builder.id(format!("_IMPL_DESERIALIZE_FOR_{}", item.ident));

    quote_item!(cx,
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
    ).unwrap()
}

// All the generics in the input, plus a bound `T: Deserialize` for each generic
// field type that will be deserialized by us, plus a bound `T: Default` for
// each generic field type that will be set to a default value.
fn build_impl_generics(
    builder: &aster::AstBuilder,
    item: &item::Item,
) -> ast::Generics {
    let generics = bound::without_defaults(item.generics);

    let generics = bound::with_where_predicates_from_fields(
        builder, item, &generics,
        |attrs| attrs.de_bound());

    match item.attrs.de_bound() {
        Some(predicates) => {
            bound::with_where_predicates(builder, &generics, predicates)
        }
        None => {
            let generics = bound::with_bound(builder, item, &generics,
                needs_deserialize_bound,
                &builder.path().ids(&["_serde", "de", "Deserialize"]).build());
            let generics = bound::with_bound(builder, item, &generics,
                requires_default,
                &builder.path().global().ids(&["std", "default", "Default"]).build());
            generics
        }
    }
}

// Fields with a `skip_deserializing` or `deserialize_with` attribute are not
// deserialized by us so we do not generate a bound. Fields with a `bound`
// attribute specify their own bound so we do not generate one. All other fields
// may need a `T: Deserialize` bound where T is the type of the field.
fn needs_deserialize_bound(attrs: &attr::Field) -> bool {
    !attrs.skip_deserializing()
        && attrs.deserialize_with().is_none()
        && attrs.de_bound().is_none()
}

// Fields with a `default` attribute (not `default=...`), and fields with a
// `skip_deserializing` attribute that do not also have `default=...`.
fn requires_default(attrs: &attr::Field) -> bool {
    attrs.default() == &attr::FieldDefault::Default
}

fn deserialize_body(
    cx: &ExtCtxt,
    builder: &aster::AstBuilder,
    item: &item::Item,
    impl_generics: &ast::Generics,
    ty: P<ast::Ty>,
) -> P<ast::Expr> {
    match item.body {
        item::Body::Enum(ref variants) => {
            deserialize_item_enum(
                cx,
                builder,
                item.ident,
                impl_generics,
                ty,
                variants,
                &item.attrs)
        }
        item::Body::Struct(item::Style::Struct, ref fields) => {
            if fields.iter().any(|field| field.ident.is_none()) {
                cx.span_bug(item.span, "struct has unnamed fields")
            }

            deserialize_struct(
                cx,
                builder,
                item.ident,
                None,
                impl_generics,
                ty,
                fields,
                &item.attrs)
        }
        item::Body::Struct(item::Style::Tuple, ref fields) |
        item::Body::Struct(item::Style::Newtype, ref fields) => {
            if fields.iter().any(|field| field.ident.is_some()) {
                cx.span_bug(item.span, "tuple struct has named fields")
            }

            deserialize_tuple(
                cx,
                builder,
                item.ident,
                None,
                impl_generics,
                ty,
                fields,
                &item.attrs)
        }
        item::Body::Struct(item::Style::Unit, _) => {
            deserialize_unit_struct(
                cx,
                builder,
                item.ident,
                &item.attrs)
        }
    }
}

// Build `__Visitor<A, B, ...>(PhantomData<A>, PhantomData<B>, ...)`
fn deserialize_visitor(
    builder: &aster::AstBuilder,
    trait_generics: &ast::Generics,
    forward_ty_params: Vec<ast::TyParam>,
    forward_tys: Vec<P<ast::Ty>>
) -> (P<ast::Item>, P<ast::Ty>, P<ast::Expr>, ast::Generics) {
    if trait_generics.ty_params.is_empty() && forward_tys.is_empty() {
        (
            builder.item().tuple_struct("__Visitor").build(),
            builder.ty().id("__Visitor"),
            builder.expr().id("__Visitor"),
            trait_generics.clone(),
        )
    } else {
        let placeholders : Vec<_> = trait_generics.ty_params.iter()
            .map(|t| builder.ty().id(t.ident))
            .collect();
        let mut trait_generics = trait_generics.clone();
        let mut ty_params = forward_ty_params.clone();
        ty_params.extend(trait_generics.ty_params.into_vec());
        trait_generics.ty_params = P::from_vec(ty_params);

        (
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
        )
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
    builder: &aster::AstBuilder,
    type_ident: Ident,
    item_attrs: &attr::Item,
) -> P<ast::Expr> {
    let type_name = name_expr(builder, item_attrs.name());

    quote_expr!(cx, {
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
    })
}

fn deserialize_tuple(
    cx: &ExtCtxt,
    builder: &aster::AstBuilder,
    type_ident: Ident,
    variant_ident: Option<Ident>,
    impl_generics: &ast::Generics,
    ty: P<ast::Ty>,
    fields: &[item::Field],
    item_attrs: &attr::Item,
) -> P<ast::Expr> {
    let where_clause = &impl_generics.where_clause;

    let (visitor_item, visitor_ty, visitor_expr, visitor_generics) = deserialize_visitor(
        builder,
        impl_generics,
        vec![deserializer_ty_param(builder)],
        vec![deserializer_ty_arg(builder)],
    );

    let is_enum = variant_ident.is_some();
    let type_path = match variant_ident {
        Some(variant_ident) => builder.path().id(type_ident).id(variant_ident).build(),
        None => builder.path().id(type_ident).build(),
    };

    let nfields = fields.len();

    let visit_newtype_struct = if !is_enum && nfields == 1 {
        Some(deserialize_newtype_struct(
            cx,
            builder,
            type_ident,
            &type_path,
            impl_generics,
            &fields[0],
        ))
    } else {
        None
    };

    let visit_seq_expr = deserialize_seq(
        cx,
        builder,
        type_ident,
        type_path,
        impl_generics,
        fields,
        false,
    );

    let dispatch = if is_enum {
        quote_expr!(cx,
            visitor.visit_tuple($nfields, $visitor_expr))
    } else if nfields == 1 {
        let type_name = name_expr(builder, item_attrs.name());
        quote_expr!(cx,
            deserializer.deserialize_newtype_struct($type_name, $visitor_expr))
    } else {
        let type_name = name_expr(builder, item_attrs.name());
        quote_expr!(cx,
            deserializer.deserialize_tuple_struct($type_name, $nfields, $visitor_expr))
    };

    quote_expr!(cx, {
        $visitor_item

        impl $visitor_generics _serde::de::Visitor for $visitor_ty $where_clause {
            type Value = $ty;

            $visit_newtype_struct

            #[inline]
            fn visit_seq<__V>(&mut self, mut visitor: __V) -> ::std::result::Result<$ty, __V::Error>
                where __V: _serde::de::SeqVisitor,
            {
                $visit_seq_expr
            }
        }

        $dispatch
    })
}

fn deserialize_seq(
    cx: &ExtCtxt,
    builder: &aster::AstBuilder,
    type_ident: Ident,
    type_path: ast::Path,
    impl_generics: &ast::Generics,
    fields: &[item::Field],
    is_struct: bool,
) -> P<ast::Expr> {
    let let_values: Vec<_> = fields.iter()
        .enumerate()
        .map(|(i, field)| {
            let name = builder.id(format!("__field{}", i));
            if field.attrs.skip_deserializing() {
                let default = expr_is_missing(cx, builder, &field.attrs);
                quote_stmt!(cx,
                    let $name = $default;
                ).unwrap()
            } else {
                let visit = match field.attrs.deserialize_with() {
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

    let result = if is_struct {
        builder.expr().struct_path(type_path)
            .with_id_exprs(
                fields.iter()
                    .enumerate()
                    .map(|(i, field)| {
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
            .build()
    } else {
        builder.expr().call()
            .build_path(type_path)
            .with_args((0..fields.len()).map(|i| builder.expr().id(format!("__field{}", i))))
            .build()
    };

    quote_expr!(cx, {
        $let_values

        try!(visitor.end());

        Ok($result)
    })
}

fn deserialize_newtype_struct(
    cx: &ExtCtxt,
    builder: &aster::AstBuilder,
    type_ident: Ident,
    type_path: &ast::Path,
    impl_generics: &ast::Generics,
    field: &item::Field,
) -> Vec<ast::TokenTree> {
    let value = match field.attrs.deserialize_with() {
        None => {
            let field_ty = &field.ty;
            quote_expr!(cx,
                try!(<$field_ty as _serde::Deserialize>::deserialize(__e)))
        }
        Some(path) => {
            let (wrapper, wrapper_impl, wrapper_ty) = wrap_deserialize_with(
                cx, builder, type_ident, impl_generics, &field.ty, path);
            quote_expr!(cx, {
                $wrapper
                $wrapper_impl
                try!(<$wrapper_ty as _serde::Deserialize>::deserialize(__e)).value
            })
        }
    };
    quote_tokens!(cx,
        #[inline]
        fn visit_newtype_struct<__E>(&mut self, __e: &mut __E) -> ::std::result::Result<Self::Value, __E::Error>
            where __E: _serde::de::Deserializer,
        {
            Ok($type_path($value))
        }
    )
}

fn deserialize_struct(
    cx: &ExtCtxt,
    builder: &aster::AstBuilder,
    type_ident: Ident,
    variant_ident: Option<Ident>,
    impl_generics: &ast::Generics,
    ty: P<ast::Ty>,
    fields: &[item::Field],
    item_attrs: &attr::Item,
) -> P<ast::Expr> {
    let where_clause = &impl_generics.where_clause;

    let (visitor_item, visitor_ty, visitor_expr, visitor_generics) = deserialize_visitor(
        builder,
        &impl_generics,
        vec![deserializer_ty_param(builder)],
        vec![deserializer_ty_arg(builder)],
    );

    let type_path = match variant_ident {
        Some(variant_ident) => builder.path().id(type_ident).id(variant_ident).build(),
        None => builder.path().id(type_ident).build(),
    };

    let visit_seq_expr = deserialize_seq(
        cx,
        builder,
        type_ident,
        type_path.clone(),
        impl_generics,
        fields,
        true,
    );

    let (field_visitor, fields_stmt, visit_map_expr) = deserialize_struct_visitor(
        cx,
        builder,
        type_ident,
        type_path.clone(),
        impl_generics,
        fields,
        item_attrs,
    );

    let is_enum = variant_ident.is_some();
    let dispatch = if is_enum {
        quote_expr!(cx,
            visitor.visit_struct(FIELDS, $visitor_expr))
    } else {
        let type_name = name_expr(builder, item_attrs.name());
        quote_expr!(cx,
            deserializer.deserialize_struct($type_name, FIELDS, $visitor_expr))
    };

    quote_expr!(cx, {
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

        $dispatch
    })
}

fn deserialize_item_enum(
    cx: &ExtCtxt,
    builder: &aster::AstBuilder,
    type_ident: Ident,
    impl_generics: &ast::Generics,
    ty: P<ast::Ty>,
    variants: &[item::Variant],
    item_attrs: &attr::Item
) -> P<ast::Expr> {
    let where_clause = &impl_generics.where_clause;

    let type_name = name_expr(builder, item_attrs.name());

    let variant_visitor = deserialize_field_visitor(
        cx,
        builder,
        variants.iter()
            .map(|variant| variant.attrs.name().deserialize_name())
            .collect(),
        item_attrs,
        true,
    );

    let variants_expr = builder.expr().ref_().slice()
        .with_exprs(
            variants.iter().map(|variant| builder.expr().str(variant.ident))
        )
        .build();

    let variants_stmt = quote_stmt!(cx,
        const VARIANTS: &'static [&'static str] = $variants_expr;
    ).unwrap();

    let ignored_arm = if item_attrs.deny_unknown_fields() {
        None
    } else {
        Some(quote_arm!(cx, __Field::__ignore => { Err(_serde::de::Error::end_of_stream()) }))
    };

    // Match arms to extract a variant from a string
    let mut variant_arms = vec![];
    for (i, variant) in variants.iter().enumerate() {
        let variant_name = builder.pat().path()
            .id("__Field").id(format!("__field{}", i))
            .build();

        let expr = deserialize_variant(
            cx,
            builder,
            type_ident,
            impl_generics,
            ty.clone(),
            variant,
            item_attrs,
        );

        let arm = quote_arm!(cx, $variant_name => { $expr });
        variant_arms.push(arm);
    }
    variant_arms.extend(ignored_arm.into_iter());

    let (visitor_item, visitor_ty, visitor_expr, visitor_generics) = deserialize_visitor(
        builder,
        impl_generics,
        vec![deserializer_ty_param(builder)],
        vec![deserializer_ty_arg(builder)],
    );

    quote_expr!(cx, {
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
    })
}

fn deserialize_variant(
    cx: &ExtCtxt,
    builder: &aster::AstBuilder,
    type_ident: Ident,
    generics: &ast::Generics,
    ty: P<ast::Ty>,
    variant: &item::Variant,
    item_attrs: &attr::Item,
) -> P<ast::Expr> {
    let variant_ident = variant.ident;

    match variant.style {
        item::Style::Unit => {
            quote_expr!(cx, {
                try!(visitor.visit_unit());
                Ok($type_ident::$variant_ident)
            })
        }
        item::Style::Newtype => {
            deserialize_newtype_variant(
                cx,
                builder,
                type_ident,
                variant_ident,
                generics,
                &variant.fields[0],
            )
        }
        item::Style::Tuple => {
            deserialize_tuple(
                cx,
                builder,
                type_ident,
                Some(variant_ident),
                generics,
                ty,
                &variant.fields,
                item_attrs,
            )
        }
        item::Style::Struct => {
            deserialize_struct(
                cx,
                builder,
                type_ident,
                Some(variant_ident),
                generics,
                ty,
                &variant.fields,
                item_attrs,
            )
        }
    }
}

fn deserialize_newtype_variant(
    cx: &ExtCtxt,
    builder: &aster::AstBuilder,
    type_ident: Ident,
    variant_ident: Ident,
    impl_generics: &ast::Generics,
    field: &item::Field,
) -> P<ast::Expr> {
    let visit = match field.attrs.deserialize_with() {
        None => {
            let field_ty = &field.ty;
            quote_expr!(cx, try!(visitor.visit_newtype::<$field_ty>()))
        }
        Some(path) => {
            let (wrapper, wrapper_impl, wrapper_ty) = wrap_deserialize_with(
                cx, builder, type_ident, impl_generics, &field.ty, path);
            quote_expr!(cx, {
                $wrapper
                $wrapper_impl
                try!(visitor.visit_newtype::<$wrapper_ty>()).value
            })
        }
    };
    quote_expr!(cx, Ok($type_ident::$variant_ident($visit)))
}

fn deserialize_field_visitor(
    cx: &ExtCtxt,
    builder: &aster::AstBuilder,
    field_names: Vec<InternedString>,
    item_attrs: &attr::Item,
    is_variant: bool,
) -> Vec<P<ast::Item>> {
    // Create the field names for the fields.
    let field_idents: Vec<_> = (0 .. field_names.len())
        .map(|i| builder.id(format!("__field{}", i)))
        .collect();

    let ignore_variant = if item_attrs.deny_unknown_fields() {
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

    let fallthrough_index_arm_expr = if !is_variant && !item_attrs.deny_unknown_fields() {
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

    let fallthrough_str_arm_expr = if !is_variant && !item_attrs.deny_unknown_fields() {
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

    let fallthrough_bytes_arm_expr = if !is_variant && !item_attrs.deny_unknown_fields() {
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
    fields: &[item::Field],
    item_attrs: &attr::Item,
) -> (Vec<P<ast::Item>>, ast::Stmt, P<ast::Expr>) {
    let field_exprs = fields.iter()
        .map(|field| field.attrs.name().deserialize_name())
        .collect();

    let field_visitor = deserialize_field_visitor(
        cx,
        builder,
        field_exprs,
        item_attrs,
        false,
    );

    let visit_map_expr = deserialize_map(
        cx,
        builder,
        type_ident,
        struct_path,
        impl_generics,
        fields,
        item_attrs,
    );

    let fields_expr = builder.expr().ref_().slice()
        .with_exprs(
            fields.iter()
                .map(|field| {
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

    (field_visitor, fields_stmt, visit_map_expr)
}

fn deserialize_map(
    cx: &ExtCtxt,
    builder: &aster::AstBuilder,
    type_ident: Ident,
    struct_path: ast::Path,
    impl_generics: &ast::Generics,
    fields: &[item::Field],
    item_attrs: &attr::Item,
) -> P<ast::Expr> {
    // Create the field names for the fields.
    let fields_names = fields.iter()
        .enumerate()
        .map(|(i, field)|
             (field, builder.id(format!("__field{}", i))))
        .collect::<Vec<_>>();

    // Declare each field that will be deserialized.
    let let_values: Vec<ast::Stmt> = fields_names.iter()
        .filter(|&&(field, _)| !field.attrs.skip_deserializing())
        .map(|&(field, name)| {
            let field_ty = &field.ty;
            quote_stmt!(cx, let mut $name: Option<$field_ty> = None;).unwrap()
        })
        .collect();

    // Match arms to extract a value for a field.
    let value_arms = fields_names.iter()
        .filter(|&&(field, _)| !field.attrs.skip_deserializing())
        .map(|&(ref field, name)| {
            let deser_name = field.attrs.name().deserialize_name();
            let name_str = builder.expr().lit().str(deser_name);

            let visit = match field.attrs.deserialize_with() {
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
    let skipped_arms = fields_names.iter()
        .filter(|&&(field, _)| field.attrs.skip_deserializing())
        .map(|&(_, name)| {
            quote_arm!(cx,
                __Field::$name => {
                    try!(visitor.visit_value::<_serde::de::impls::IgnoredAny>());
                }
            )
        })
        .collect::<Vec<_>>();

    // Visit ignored values to consume them
    let ignored_arm = if item_attrs.deny_unknown_fields() {
        None
    } else {
        Some(quote_arm!(cx,
            _ => { try!(visitor.visit_value::<_serde::de::impls::IgnoredAny>()); }
        ))
    };

    let extract_values = fields_names.iter()
        .filter(|&&(field, _)| !field.attrs.skip_deserializing())
        .map(|&(field, name)| {
            let missing_expr = expr_is_missing(cx, builder, &field.attrs);

            quote_stmt!(cx,
                let $name = match $name {
                    Some($name) => $name,
                    None => $missing_expr
                };
            ).unwrap()
        })
        .collect::<Vec<_>>();

    let result = builder.expr().struct_path(struct_path)
        .with_id_exprs(
            fields_names.iter()
                .map(|&(field, name)| {
                    (
                        match field.ident {
                            Some(name) => name.clone(),
                            None => {
                                cx.span_bug(field.span, "struct contains unnamed fields")
                            }
                        },
                        if field.attrs.skip_deserializing() {
                            expr_is_missing(cx, builder, &field.attrs)
                        } else {
                            builder.expr().id(name)
                        }
                    )
                })
        )
        .build();

    quote_expr!(cx, {
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
    })
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
    builder: &aster::AstBuilder,
    attrs: &attr::Field,
) -> P<ast::Expr> {
    match *attrs.default() {
        attr::FieldDefault::Default => {
            return quote_expr!(cx, ::std::default::Default::default());
        }
        attr::FieldDefault::Path(ref path) => {
            return quote_expr!(cx, $path());
        }
        attr::FieldDefault::None => { /* below */ }
    }

    let name = name_expr(builder, attrs.name());
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

fn name_expr(
    builder: &aster::AstBuilder,
    name: &attr::Name,
) -> P<ast::Expr> {
    builder.expr().str(name.deserialize_name())
}

fn check_no_str(
    cx: &ExtCtxt,
    item: &item::Item,
) -> Result<(), Error> {
    let fail = |field: &item::Field| {
        cx.span_err(
            field.span,
            "Serde does not support deserializing fields of type &str; \
             consider using String instead");
        Err(Error)
    };

    for field in item.body.all_fields() {
        if field.attrs.skip_deserializing()
            || field.attrs.deserialize_with().is_some() { continue }

        if let ast::TyKind::Rptr(_, ref inner) = field.ty.node {
            if let ast::TyKind::Path(_, ref path) = inner.ty.node {
                if path.segments.len() == 1
                    && path.segments[0].identifier.name.as_str() == "str"
                {
                    return fail(field);
                }
            }
        }
    }
    Ok(())
}
