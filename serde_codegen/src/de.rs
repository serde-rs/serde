use std::collections::HashSet;

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
use syntax::ptr::P;

use attr::{self, ContainerAttrs};
use field;

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
                "`derive` may only be applied to structs and enums");
            return;
        }
    };

    let builder = aster::AstBuilder::new().span(span);

    let generics = match item.node {
        ast::ItemStruct(_, ref generics) => generics,
        ast::ItemEnum(_, ref generics) => generics,
        _ => cx.bug("expected ItemStruct or ItemEnum in #[derive(Deserialize)]")
    };

    let impl_generics = builder.from_generics(generics.clone())
        .add_ty_param_bound(
            builder.path().global().ids(&["serde", "de", "Deserialize"]).build()
        )
        .build();

    let ty = builder.ty().path()
        .segment(item.ident).with_generics(impl_generics.clone()).build()
        .build();

    let body = deserialize_body(
        cx,
        &builder,
        &item,
        &impl_generics,
        ty.clone(),
    );

    let where_clause = &impl_generics.where_clause;

    let impl_item = quote_item!(cx,
        impl $impl_generics ::serde::de::Deserialize for $ty $where_clause {
            fn deserialize<__D>(deserializer: &mut __D) -> ::std::result::Result<$ty, __D::Error>
                where __D: ::serde::de::Deserializer,
            {
                $body
            }
        }
    ).unwrap();

    push(Annotatable::Item(impl_item))
}

fn deserialize_body(
    cx: &ExtCtxt,
    builder: &aster::AstBuilder,
    item: &Item,
    impl_generics: &ast::Generics,
    ty: P<ast::Ty>,
) -> P<ast::Expr> {
    let container_attrs = field::container_attrs(cx, item);

    match item.node {
        ast::ItemStruct(ref variant_data, _) => {
            deserialize_item_struct(
                cx,
                builder,
                item,
                impl_generics,
                ty,
                variant_data,
                &container_attrs,
            )
        }
        ast::ItemEnum(ref enum_def, _) => {
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
        _ => cx.bug("expected ItemStruct or ItemEnum in #[derive(Deserialize)]")
    }
}

fn deserialize_item_struct(
    cx: &ExtCtxt,
    builder: &aster::AstBuilder,
    item: &Item,
    impl_generics: &ast::Generics,
    ty: P<ast::Ty>,
    variant_data: &ast::VariantData,
    container_attrs: &ContainerAttrs,
) -> P<ast::Expr> {
    match *variant_data {
        ast::VariantData::Unit(_) => {
            deserialize_unit_struct(
                cx,
                &builder,
                item.ident,
            )
        }
        ast::VariantData::Tuple(ref fields, _) if fields.len() == 1 => {
            deserialize_newtype_struct(
                cx,
                &builder,
                item.ident,
                impl_generics,
                ty,
            )
        }
        ast::VariantData::Tuple(ref fields, _) => {
            if fields.iter().any(|field| !field.node.kind.is_unnamed()) {
                cx.bug("tuple struct has named fields")
            }

            deserialize_tuple_struct(
                cx,
                &builder,
                item.ident,
                impl_generics,
                ty,
                fields.len(),
            )
        }
        ast::VariantData::Struct(ref fields, _) => {
            if fields.iter().any(|field| field.node.kind.is_unnamed()) {
                cx.bug("struct has unnamed fields")
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
                     .global()
                     .segment("serde").build()
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
) -> P<ast::Expr> {
    let type_name = builder.expr().str(type_ident);

    quote_expr!(cx, {
        struct __Visitor;

        impl ::serde::de::Visitor for __Visitor {
            type Value = $type_ident;

            #[inline]
            fn visit_unit<E>(&mut self) -> ::std::result::Result<$type_ident, E>
                where E: ::serde::de::Error,
            {
                Ok($type_ident)
            }

            #[inline]
            fn visit_seq<V>(&mut self, mut visitor: V) -> ::std::result::Result<$type_ident, V::Error>
                where V: ::serde::de::SeqVisitor,
            {
                try!(visitor.end());
                self.visit_unit()
            }
        }

        deserializer.deserialize_unit_struct($type_name, __Visitor)
    })
}

fn deserialize_newtype_struct(
    cx: &ExtCtxt,
    builder: &aster::AstBuilder,
    type_ident: Ident,
    impl_generics: &ast::Generics,
    ty: P<ast::Ty>,
) -> P<ast::Expr> {
    let where_clause = &impl_generics.where_clause;

    let (visitor_item, visitor_ty, visitor_expr, visitor_generics) =
        deserialize_visitor(
            builder,
            impl_generics,
            vec![deserializer_ty_param(builder)],
            vec![deserializer_ty_arg(builder)],
        );

    let visit_seq_expr = deserialize_seq(
        cx,
        builder,
        builder.path().id(type_ident).build(),
        1,
    );

    let type_name = builder.expr().str(type_ident);

    quote_expr!(cx, {
        $visitor_item

        impl $visitor_generics ::serde::de::Visitor for $visitor_ty $where_clause {
            type Value = $ty;

            #[inline]
            fn visit_newtype_struct<D>(&mut self, deserializer: &mut D) -> ::std::result::Result<Self::Value, D::Error>
                where D: ::serde::de::Deserializer,
            {
                let value = try!(::serde::de::Deserialize::deserialize(deserializer));
                Ok($type_ident(value))
            }

            #[inline]
            fn visit_seq<__V>(&mut self, mut visitor: __V) -> ::std::result::Result<$ty, __V::Error>
                where __V: ::serde::de::SeqVisitor,
            {
                $visit_seq_expr
            }
        }

        deserializer.deserialize_newtype_struct($type_name, $visitor_expr)
    })
}

fn deserialize_tuple_struct(
    cx: &ExtCtxt,
    builder: &aster::AstBuilder,
    type_ident: Ident,
    impl_generics: &ast::Generics,
    ty: P<ast::Ty>,
    fields: usize,
) -> P<ast::Expr> {
    let where_clause = &impl_generics.where_clause;

    let (visitor_item, visitor_ty, visitor_expr, visitor_generics) =
        deserialize_visitor(
            builder,
            impl_generics,
            vec![deserializer_ty_param(builder)],
            vec![deserializer_ty_arg(builder)],
        );

    let visit_seq_expr = deserialize_seq(
        cx,
        builder,
        builder.path().id(type_ident).build(),
        fields,
    );

    let type_name = builder.expr().str(type_ident);

    quote_expr!(cx, {
        $visitor_item

        impl $visitor_generics ::serde::de::Visitor for $visitor_ty $where_clause {
            type Value = $ty;

            #[inline]
            fn visit_seq<__V>(&mut self, mut visitor: __V) -> ::std::result::Result<$ty, __V::Error>
                where __V: ::serde::de::SeqVisitor,
            {
                $visit_seq_expr
            }
        }

        deserializer.deserialize_tuple_struct($type_name, $fields, $visitor_expr)
    })
}

fn deserialize_seq(
    cx: &ExtCtxt,
    builder: &aster::AstBuilder,
    struct_path: ast::Path,
    fields: usize,
) -> P<ast::Expr> {
    let let_values: Vec<P<ast::Stmt>> = (0 .. fields)
        .map(|i| {
            let name = builder.id(format!("__field{}", i));
            quote_stmt!(cx,
                let $name = match try!(visitor.visit()) {
                    Some(value) => { value },
                    None => {
                        return Err(::serde::de::Error::end_of_stream());
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
    struct_path: ast::Path,
    fields: &[ast::StructField],
) -> P<ast::Expr> {
    let let_values: Vec<P<ast::Stmt>> = (0 .. fields.len())
        .map(|i| {
            let name = builder.id(format!("__field{}", i));
            quote_stmt!(cx,
                let $name = match try!(visitor.visit()) {
                    Some(value) => { value },
                    None => {
                        return Err(::serde::de::Error::end_of_stream());
                    }
                };
            ).unwrap()
        })
        .collect();

    let result = builder.expr().struct_path(struct_path)
        .with_id_exprs(
            fields.iter()
                .enumerate()
                .map(|(i, field)| {
                    (
                        match field.node.kind {
                            ast::NamedField(name, _) => name.clone(),
                            ast::UnnamedField(_) => cx.bug("struct contains unnamed fields"),
                        },
                        builder.expr().id(format!("__field{}", i)),
                    )
                })
        )
        .build();

    quote_expr!(cx, {
        $let_values

        try!(visitor.end());

        Ok($result)
    })
}

fn deserialize_struct(
    cx: &ExtCtxt,
    builder: &aster::AstBuilder,
    type_ident: Ident,
    impl_generics: &ast::Generics,
    ty: P<ast::Ty>,
    fields: &[ast::StructField],
    container_attrs: &ContainerAttrs,
) -> P<ast::Expr> {
    let where_clause = &impl_generics.where_clause;

    let (visitor_item, visitor_ty, visitor_expr, visitor_generics) = deserialize_visitor(
        builder,
        &impl_generics,
        vec![deserializer_ty_param(builder)],
        vec![deserializer_ty_arg(builder)],
    );

    let type_path = builder.path().id(type_ident).build();

    let visit_seq_expr = deserialize_struct_as_seq(
        cx,
        builder,
        type_path.clone(),
        fields,
    );

    let (field_visitor, fields_stmt, visit_map_expr) = deserialize_struct_visitor(
        cx,
        builder,
        type_path.clone(),
        fields,
        container_attrs
    );

    let type_name = builder.expr().str(type_ident);

    quote_expr!(cx, {
        $field_visitor

        $visitor_item

        impl $visitor_generics ::serde::de::Visitor for $visitor_ty $where_clause {
            type Value = $ty;

            #[inline]
            fn visit_seq<__V>(&mut self, mut visitor: __V) -> ::std::result::Result<$ty, __V::Error>
                where __V: ::serde::de::SeqVisitor,
            {
                $visit_seq_expr
            }

            #[inline]
            fn visit_map<__V>(&mut self, mut visitor: __V) -> ::std::result::Result<$ty, __V::Error>
                where __V: ::serde::de::MapVisitor,
            {
                $visit_map_expr
            }
        }

        $fields_stmt

        deserializer.deserialize_struct($type_name, FIELDS, $visitor_expr)
    })
}

fn deserialize_item_enum(
    cx: &ExtCtxt,
    builder: &aster::AstBuilder,
    type_ident: Ident,
    impl_generics: &ast::Generics,
    ty: P<ast::Ty>,
    enum_def: &EnumDef,
    container_attrs: &ContainerAttrs
) -> P<ast::Expr> {
    let where_clause = &impl_generics.where_clause;

    let type_name = builder.expr().str(type_ident);

    let variant_visitor = deserialize_field_visitor(
        cx,
        builder,
        enum_def.variants.iter()
            .map(|variant| {
                let expr = builder.expr().str(variant.node.name);
                 attr::FieldAttrsBuilder::new(builder)
                    .name(expr)
                    .default()
                    .build()
            })
            .collect(),
        container_attrs,
    );

    let variants_expr = builder.expr().addr_of().slice()
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

    let ignored_arm = if !container_attrs.disallow_unknown() {
        Some(quote_arm!(cx, __Field::__ignore => { Err(::serde::de::Error::end_of_stream()) }))
    } else {
        None
    };

    // Match arms to extract a variant from a string
    let variant_arms: Vec<_> = enum_def.variants.iter()
        .enumerate()
        .map(|(i, variant)| {
            let variant_name = builder.pat().enum_()
                .id("__Field").id(format!("__field{}", i)).build()
                .build();

            let expr = deserialize_variant(
                cx,
                builder,
                type_ident,
                impl_generics,
                ty.clone(),
                variant,
                container_attrs,
            );

            quote_arm!(cx, $variant_name => { $expr })
        })
        .chain(ignored_arm.into_iter())
        .collect();

    let (visitor_item, visitor_ty, visitor_expr, visitor_generics) =
        deserialize_visitor(
            builder,
            impl_generics,
            vec![deserializer_ty_param(builder)],
            vec![deserializer_ty_arg(builder)],
        );

    quote_expr!(cx, {
        $variant_visitor

        $visitor_item

        impl $visitor_generics ::serde::de::EnumVisitor for $visitor_ty $where_clause {
            type Value = $ty;

            fn visit<__V>(&mut self, mut visitor: __V) -> ::std::result::Result<$ty, __V::Error>
                where __V: ::serde::de::VariantVisitor,
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
    variant: &ast::Variant,
    container_attrs: &ContainerAttrs,
) -> P<ast::Expr> {
    let variant_ident = variant.node.name;

    match variant.node.data {
        ast::VariantData::Unit(_) => {
            quote_expr!(cx, {
                try!(visitor.visit_unit());
                Ok($type_ident::$variant_ident)
            })
        }
        ast::VariantData::Tuple(ref args, _) if args.len() == 1 => {
            quote_expr!(cx, {
                let val = try!(visitor.visit_newtype());
                Ok($type_ident::$variant_ident(val))
            })
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
) -> P<ast::Expr> {
    let where_clause = &generics.where_clause;

    let (visitor_item, visitor_ty, visitor_expr, visitor_generics) =
        deserialize_visitor(
            builder,
            generics,
            vec![deserializer_ty_param(builder)],
            vec![deserializer_ty_arg(builder)],
        );

    let visit_seq_expr = deserialize_seq(
        cx,
        builder,
        builder.path().id(type_ident).id(variant_ident).build(),
        fields,
    );

    quote_expr!(cx, {
        $visitor_item

        impl $visitor_generics ::serde::de::Visitor for $visitor_ty $where_clause {
            type Value = $ty;

            fn visit_seq<__V>(&mut self, mut visitor: __V) -> ::std::result::Result<$ty, __V::Error>
                where __V: ::serde::de::SeqVisitor,
            {
                $visit_seq_expr
            }
        }

        visitor.visit_tuple($fields, $visitor_expr)
    })
}

fn deserialize_struct_variant(
    cx: &ExtCtxt,
    builder: &aster::AstBuilder,
    type_ident: ast::Ident,
    variant_ident: ast::Ident,
    generics: &ast::Generics,
    ty: P<ast::Ty>,
    fields: &[ast::StructField],
    container_attrs: &ContainerAttrs,
) -> P<ast::Expr> {
    let where_clause = &generics.where_clause;

    let type_path = builder.path()
        .id(type_ident)
        .id(variant_ident)
        .build();

    let visit_seq_expr = deserialize_struct_as_seq(
        cx,
        builder,
        type_path.clone(),
        fields,
    );

    let (field_visitor, fields_stmt, field_expr) = deserialize_struct_visitor(
        cx,
        builder,
        type_path,
        fields,
        container_attrs,
    );

    let (visitor_item, visitor_ty, visitor_expr, visitor_generics) =
        deserialize_visitor(
            builder,
            generics,
            vec![deserializer_ty_param(builder)],
            vec![deserializer_ty_arg(builder)],
        );

    quote_expr!(cx, {
        $field_visitor

        $visitor_item

        impl $visitor_generics ::serde::de::Visitor for $visitor_ty $where_clause {
            type Value = $ty;

            #[inline]
            fn visit_seq<__V>(&mut self, mut visitor: __V) -> ::std::result::Result<$ty, __V::Error>
                where __V: ::serde::de::SeqVisitor,
            {
                $visit_seq_expr
            }

            #[inline]
            fn visit_map<__V>(&mut self, mut visitor: __V) -> ::std::result::Result<$ty, __V::Error>
                where __V: ::serde::de::MapVisitor,
            {
                $field_expr
            }
        }

        $fields_stmt

        visitor.visit_struct(FIELDS, $visitor_expr)
    })
}

fn deserialize_field_visitor(
    cx: &ExtCtxt,
    builder: &aster::AstBuilder,
    field_attrs: Vec<attr::FieldAttrs>,
    container_attrs: &ContainerAttrs,
) -> Vec<P<ast::Item>> {
    // Create the field names for the fields.
    let field_idents: Vec<ast::Ident> = (0 .. field_attrs.len())
        .map(|i| builder.id(format!("__field{}", i)))
        .collect();

    let ignore_variant = if !container_attrs.disallow_unknown() {
        let skip_ident = builder.id("__ignore");
        Some(builder.variant(skip_ident).unit())
    } else {
        None
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

    let index_body = quote_expr!(cx,
        match value {
            $index_field_arms
            _ => { Err(::serde::de::Error::syntax("expected a field")) }
        }
    );

    // A set of all the formats that have specialized field attributes
    let formats = field_attrs.iter()
        .fold(HashSet::new(), |mut set, field_expr| {
            set.extend(field_expr.formats());
            set
        });

    // Match arms to extract a field from a string
    let default_field_arms: Vec<_> = field_idents.iter()
        .zip(field_attrs.iter())
        .map(|(field_ident, field_expr)| {
            let expr = field_expr.default_key_expr();
            quote_arm!(cx, $expr => { Ok(__Field::$field_ident) })
        })
        .collect();

    let fallthrough_arm_expr = if !container_attrs.disallow_unknown() {
        quote_expr!(cx, Ok(__Field::__ignore))
    } else {
        quote_expr!(cx, Err(::serde::de::Error::unknown_field(value)))
    };

    let str_body = if formats.is_empty() {
        // No formats specific attributes, so no match on format required
        quote_expr!(cx,
            match value {
                $default_field_arms
                _ => { $fallthrough_arm_expr }
            })
    } else {
        let field_arms: Vec<_> = formats.iter()
            .map(|fmt| {
                field_idents.iter()
                    .zip(field_attrs.iter())
                    .map(|(field_ident, field_expr)| {
                        let expr = field_expr.key_expr(fmt);
                        quote_arm!(cx, $expr => { Ok(__Field::$field_ident) })
                    })
                    .collect::<Vec<_>>()
            })
            .collect();

        let fmt_matches: Vec<_> = formats.iter()
            .zip(field_arms.iter())
            .map(|(ref fmt, ref arms)| {
                quote_arm!(cx, $fmt => {
                    match value {
                        $arms
                        _ => {
                            $fallthrough_arm_expr
                        }
                    }})
            })
            .collect();

        quote_expr!(cx,
            match __D::format() {
                $fmt_matches
                _ => match value {
                    $default_field_arms
                    _ => $fallthrough_arm_expr
                }
            }
        )
    };

    let impl_item = quote_item!(cx,
        impl ::serde::de::Deserialize for __Field {
            #[inline]
            fn deserialize<D>(deserializer: &mut D) -> ::std::result::Result<__Field, D::Error>
                where D: ::serde::de::Deserializer,
            {
                use std::marker::PhantomData;

                struct __FieldVisitor<D> {
                    phantom: PhantomData<D>
                }

                impl<__D> ::serde::de::Visitor for __FieldVisitor<__D>
                    where __D: ::serde::de::Deserializer
                {
                    type Value = __Field;

                    fn visit_usize<E>(&mut self, value: usize) -> ::std::result::Result<__Field, E>
                        where E: ::serde::de::Error,
                    {
                        $index_body
                    }

                    fn visit_str<E>(&mut self, value: &str) -> ::std::result::Result<__Field, E>
                        where E: ::serde::de::Error,
                    {
                        $str_body
                    }

                    fn visit_bytes<E>(&mut self, value: &[u8]) -> ::std::result::Result<__Field, E>
                        where E: ::serde::de::Error,
                    {
                        // TODO: would be better to generate a byte string literal match
                        match ::std::str::from_utf8(value) {
                            Ok(s) => self.visit_str(s),
                            _ => {
                                Err(
                                    ::serde::de::Error::syntax(
                                        "could not convert a byte string to a String"
                                    )
                                )
                            }
                        }
                    }
                }

                deserializer.deserialize(__FieldVisitor::<D>{ phantom: PhantomData })
            }
        }
    ).unwrap();

    vec![field_enum, impl_item]
}

fn deserialize_struct_visitor(
    cx: &ExtCtxt,
    builder: &aster::AstBuilder,
    struct_path: ast::Path,
    fields: &[ast::StructField],
    container_attrs: &ContainerAttrs,
) -> (Vec<P<ast::Item>>, P<ast::Stmt>, P<ast::Expr>) {
    let field_visitor = deserialize_field_visitor(
        cx,
        builder,
        field::struct_field_attrs(cx, builder, fields),
        container_attrs
    );

    let visit_map_expr = deserialize_map(
        cx,
        builder,
        struct_path,
        fields,
        container_attrs,
    );

    let fields_expr = builder.expr().addr_of().slice()
        .with_exprs(
            fields.iter()
                .map(|field| {
                    match field.node.kind {
                        ast::NamedField(name, _) => builder.expr().str(name),
                        ast::UnnamedField(_) => panic!("struct contains unnamed fields"),
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
    struct_path: ast::Path,
    fields: &[ast::StructField],
    container_attrs: &ContainerAttrs,
) -> P<ast::Expr> {
    // Create the field names for the fields.
    let field_names: Vec<ast::Ident> = (0 .. fields.len())
        .map(|i| builder.id(format!("__field{}", i)))
        .collect();

    // Declare each field.
    let let_values: Vec<P<ast::Stmt>> = field_names.iter()
        .map(|field_name| quote_stmt!(cx, let mut $field_name = None;).unwrap())
        .collect();


    // Visit ignored values to consume them
    let ignored_arm = if !container_attrs.disallow_unknown() {
        Some(quote_arm!(cx,
            _ => { try!(visitor.visit_value::<::serde::de::impls::IgnoredAny>()); }
        ))
    } else {
        None
    };

    // Match arms to extract a value for a field.
    let value_arms: Vec<ast::Arm> = field_names.iter()
        .map(|field_name| {
            quote_arm!(cx,
                __Field::$field_name => {
                    $field_name = Some(try!(visitor.visit_value()));
                }
            )
        })
        .chain(ignored_arm.into_iter())
        .collect();

    let extract_values: Vec<P<ast::Stmt>> = field_names.iter()
        .zip(field::struct_field_attrs(cx, builder, fields).iter())
        .map(|(field_name, field_attr)| {
            let missing_expr = if field_attr.use_default() {
                quote_expr!(cx, ::std::default::Default::default())
            } else {
                let formats = field_attr.formats();
                let arms : Vec<_> = formats.iter()
                    .map(|format| {
                        let key_expr = field_attr.key_expr(format);
                        quote_arm!(cx, $format => { $key_expr })
                    })
                    .collect();
                let default = field_attr.default_key_expr();
                if arms.is_empty() {
                    quote_expr!(cx, try!(visitor.missing_field($default)))
                } else {
                    quote_expr!(
                        cx,
                        try!(visitor.missing_field(
                            match __D::format() {
                                $arms
                                _ => { $default }
                            })))
                }
            };

            quote_stmt!(cx,
                let $field_name = match $field_name {
                    Some($field_name) => $field_name,
                    None => $missing_expr
                };
            ).unwrap()
        })
        .collect();

    let result = builder.expr().struct_path(struct_path)
        .with_id_exprs(
            fields.iter()
                .zip(field_names.iter())
                .map(|(field, field_name)| {
                    (
                        match field.node.kind {
                            ast::NamedField(name, _) => name.clone(),
                            ast::UnnamedField(_) => panic!("struct contains unnamed fields"),
                        },
                        builder.expr().id(field_name),
                    )
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

        try!(visitor.end());

        Ok($result)
    })
}
