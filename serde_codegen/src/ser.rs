use aster;
use reduce::Reduce;

use syntax::ast::{self, Ident, MetaItem};
use syntax::codemap::Span;
use syntax::ext::base::{Annotatable, ExtCtxt};
use syntax::ptr::P;

use attr;
use bound;
use error::Error;
use item;

pub fn expand_derive_serialize(
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
                "`#[derive(Serialize)]` may only be applied to structs and enums");
            return;
        }
    };

    let builder = aster::AstBuilder::new().span(span);

    let impl_item = match serialize_item(cx, &builder, &item) {
        Ok(item) => item,
        Err(Error) => {
            // An error occured, but it should have been reported already.
            return;
        }
    };

    push(Annotatable::Item(impl_item))
}

fn serialize_item(
    cx: &ExtCtxt,
    builder: &aster::AstBuilder,
    item: &ast::Item,
) -> Result<P<ast::Item>, Error> {
    let item = try!(item::Item::from_ast(cx, "Serialize", item));

    let impl_generics = build_impl_generics(builder, &item);

    let ty = builder.ty().path()
        .segment(item.ident).with_generics(impl_generics.clone()).build()
        .build();

    let body = serialize_body(cx,
                              builder,
                              &item,
                              &impl_generics,
                              ty.clone());

    let where_clause = &impl_generics.where_clause;

    let dummy_const = builder.id(format!("_IMPL_SERIALIZE_FOR_{}", item.ident));

    Ok(quote_item!(cx,
        #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
        const $dummy_const: () = {
            extern crate serde as _serde;
            #[automatically_derived]
            impl $impl_generics _serde::ser::Serialize for $ty $where_clause {
                fn serialize<__S>(&self, _serializer: &mut __S) -> ::std::result::Result<(), __S::Error>
                    where __S: _serde::ser::Serializer,
                {
                    $body
                }
            }
        };
    ).unwrap())
}

// All the generics in the input, plus a bound `T: Serialize` for each generic
// field type that will be serialized by us.
fn build_impl_generics(
    builder: &aster::AstBuilder,
    item: &item::Item,
) -> ast::Generics {
    let generics = bound::without_defaults(item.generics);

    let generics = bound::with_where_predicates_from_fields(
        builder, item, &generics,
        |attrs| attrs.ser_bound());

    match item.attrs.ser_bound() {
        Some(predicates) => {
            bound::with_where_predicates(builder, &generics, predicates)
        }
        None => {
            bound::with_bound(builder, item, &generics,
                needs_serialize_bound,
                &builder.path().ids(&["_serde", "ser", "Serialize"]).build())
        }
    }
}

// Fields with a `skip_serializing` or `serialize_with` attribute are not
// serialized by us so we do not generate a bound. Fields with a `bound`
// attribute specify their own bound so we do not generate one. All other fields
// may need a `T: Serialize` bound where T is the type of the field.
fn needs_serialize_bound(attrs: &attr::FieldAttrs) -> bool {
    !attrs.skip_serializing()
        && attrs.serialize_with().is_none()
        && attrs.ser_bound().is_none()
}

fn serialize_body(
    cx: &ExtCtxt,
    builder: &aster::AstBuilder,
    item: &item::Item,
    impl_generics: &ast::Generics,
    ty: P<ast::Ty>,
) -> P<ast::Expr> {
    match item.body {
        item::Body::Enum(ref variants) => {
            serialize_item_enum(
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

            serialize_struct(
                cx,
                builder,
                impl_generics,
                ty,
                fields,
                &item.attrs)
        }
        item::Body::Struct(item::Style::Tuple, ref fields) => {
            if fields.iter().any(|field| field.ident.is_some()) {
                cx.span_bug(item.span, "tuple struct has named fields")
            }

            serialize_tuple_struct(
                cx,
                builder,
                impl_generics,
                ty,
                fields,
                &item.attrs)
        }
        item::Body::Struct(item::Style::Newtype, ref fields) => {
            serialize_newtype_struct(
                cx,
                builder,
                impl_generics,
                ty,
                &fields[0],
                &item.attrs)
        }
        item::Body::Struct(item::Style::Unit, _) => {
            serialize_unit_struct(
                cx,
                &item.attrs)
        }
    }
}

fn serialize_unit_struct(
    cx: &ExtCtxt,
    container_attrs: &attr::ContainerAttrs,
) -> P<ast::Expr> {
    let type_name = container_attrs.name().serialize_name_expr();

    quote_expr!(cx,
        _serializer.serialize_unit_struct($type_name)
    )
}

fn serialize_newtype_struct(
    cx: &ExtCtxt,
    builder: &aster::AstBuilder,
    impl_generics: &ast::Generics,
    container_ty: P<ast::Ty>,
    field: &item::Field,
    container_attrs: &attr::ContainerAttrs,
) -> P<ast::Expr> {
    let type_name = container_attrs.name().serialize_name_expr();

    let mut field_expr = quote_expr!(cx, &self.0);
    if let Some(path) = field.attrs.serialize_with() {
        field_expr = wrap_serialize_with(cx, builder,
            &container_ty, impl_generics, &field.ty, path, field_expr);
    }

    quote_expr!(cx,
        _serializer.serialize_newtype_struct($type_name, $field_expr)
    )
}

fn serialize_tuple_struct(
    cx: &ExtCtxt,
    builder: &aster::AstBuilder,
    impl_generics: &ast::Generics,
    ty: P<ast::Ty>,
    fields: &[item::Field],
    container_attrs: &attr::ContainerAttrs,
) -> P<ast::Expr> {
    let (visitor_struct, visitor_impl) = serialize_tuple_struct_visitor(
        cx,
        builder,
        ty.clone(),
        builder.ty()
            .ref_()
            .lifetime("'__a")
            .build_ty(ty.clone()),
        builder.id("serialize_tuple_struct_elt"),
        fields,
        impl_generics,
        false,
    );

    let type_name = container_attrs.name().serialize_name_expr();

    quote_expr!(cx, {
        $visitor_struct
        $visitor_impl
        _serializer.serialize_tuple_struct($type_name, Visitor {
            value: self,
            state: 0,
            _structure_ty: ::std::marker::PhantomData::<&$ty>,
        })
    })
}

fn serialize_struct(
    cx: &ExtCtxt,
    builder: &aster::AstBuilder,
    impl_generics: &ast::Generics,
    ty: P<ast::Ty>,
    fields: &[item::Field],
    container_attrs: &attr::ContainerAttrs,
) -> P<ast::Expr> {
    let (visitor_struct, visitor_impl) = serialize_struct_visitor(
        cx,
        builder,
        ty.clone(),
        builder.ty()
            .ref_()
            .lifetime("'__a")
            .build_ty(ty.clone()),
        builder.id("serialize_struct_elt"),
        fields,
        impl_generics,
        false,
    );

    let type_name = container_attrs.name().serialize_name_expr();

    quote_expr!(cx, {
        $visitor_struct
        $visitor_impl
        _serializer.serialize_struct($type_name, Visitor {
            value: self,
            state: 0,
            _structure_ty: ::std::marker::PhantomData::<&$ty>,
        })
    })
}

fn serialize_item_enum(
    cx: &ExtCtxt,
    builder: &aster::AstBuilder,
    type_ident: Ident,
    impl_generics: &ast::Generics,
    ty: P<ast::Ty>,
    variants: &[item::Variant],
    container_attrs: &attr::ContainerAttrs,
) -> P<ast::Expr> {
    let arms: Vec<_> =
        variants.iter()
            .enumerate()
            .map(|(variant_index, variant)| {
                serialize_variant(
                    cx,
                    builder,
                    type_ident,
                    impl_generics,
                    ty.clone(),
                    variant,
                    variant_index,
                    container_attrs,
                )
            })
            .collect();

    quote_expr!(cx,
        match *self {
            $arms
        }
    )
}

fn serialize_variant(
    cx: &ExtCtxt,
    builder: &aster::AstBuilder,
    type_ident: Ident,
    generics: &ast::Generics,
    ty: P<ast::Ty>,
    variant: &item::Variant,
    variant_index: usize,
    container_attrs: &attr::ContainerAttrs,
) -> ast::Arm {
    let type_name = container_attrs.name().serialize_name_expr();

    let variant_ident = variant.ident;
    let variant_name = variant.attrs.name().serialize_name_expr();

    match variant.style {
        item::Style::Unit => {
            quote_arm!(cx,
                $type_ident::$variant_ident => {
                    _serde::ser::Serializer::serialize_unit_variant(
                        _serializer,
                        $type_name,
                        $variant_index,
                        $variant_name,
                    )
                }
            )
        },
        item::Style::Newtype => {
            let expr = serialize_newtype_variant(
                cx,
                builder,
                type_name,
                variant_index,
                variant_name,
                ty,
                generics,
                &variant.fields[0],
            );

            quote_arm!(cx,
                $type_ident::$variant_ident(ref __simple_value) => { $expr }
            )
        },
        item::Style::Tuple => {
            let field_names: Vec<ast::Ident> = (0 .. variant.fields.len())
                .map(|i| builder.id(format!("__field{}", i)))
                .collect();

            let pat = builder.pat().enum_()
                .id(type_ident).id(variant_ident).build()
                .with_pats(
                    field_names.iter()
                        .map(|field| builder.pat().ref_id(field))
                )
                .build();

            let expr = serialize_tuple_variant(
                cx,
                builder,
                type_name,
                variant_index,
                variant_name,
                generics,
                ty,
                &variant.fields,
                field_names,
            );

            quote_arm!(cx,
                $pat => { $expr }
            )
        }
        item::Style::Struct => {
            let field_names: Vec<_> = (0 .. variant.fields.len())
                .map(|i| builder.id(format!("__field{}", i)))
                .collect();

            let pat = builder.pat().struct_()
                .id(type_ident).id(variant_ident).build()
                .with_pats(
                    field_names.iter()
                        .zip(variant.fields.iter())
                        .map(|(id, field)| {
                            let name = match field.ident {
                                Some(name) => name,
                                None => {
                                    cx.span_bug(field.span, "struct variant has unnamed fields")
                                }
                            };

                            (name, builder.pat().ref_id(id))
                        })
                )
                .build();

            let expr = serialize_struct_variant(
                cx,
                builder,
                variant_index,
                variant_name,
                generics,
                ty,
                &variant.fields,
                field_names,
                container_attrs,
            );

            quote_arm!(cx,
                $pat => { $expr }
            )
        }
    }
}

fn serialize_newtype_variant(
    cx: &ExtCtxt,
    builder: &aster::AstBuilder,
    type_name: P<ast::Expr>,
    variant_index: usize,
    variant_name: P<ast::Expr>,
    container_ty: P<ast::Ty>,
    generics: &ast::Generics,
    field: &item::Field,
) -> P<ast::Expr> {
    let mut field_expr = quote_expr!(cx, __simple_value);
    if let Some(path) = field.attrs.serialize_with() {
        field_expr = wrap_serialize_with(cx, builder,
            &container_ty, generics, &field.ty, path, field_expr);
    }

    quote_expr!(cx,
        _serde::ser::Serializer::serialize_newtype_variant(
            _serializer,
            $type_name,
            $variant_index,
            $variant_name,
            $field_expr,
        )
    )
}

fn serialize_tuple_variant(
    cx: &ExtCtxt,
    builder: &aster::AstBuilder,
    type_name: P<ast::Expr>,
    variant_index: usize,
    variant_name: P<ast::Expr>,
    generics: &ast::Generics,
    structure_ty: P<ast::Ty>,
    fields: &[item::Field],
    field_names: Vec<Ident>,
) -> P<ast::Expr> {
    let variant_ty = builder.ty().tuple()
        .with_tys(
            fields.iter().map(|field| {
                builder.ty()
                    .ref_()
                    .lifetime("'__a")
                    .build_ty(field.ty.clone())
            })
        )
        .build();

    let (visitor_struct, visitor_impl) = serialize_tuple_struct_visitor(
        cx,
        builder,
        structure_ty.clone(),
        variant_ty,
        builder.id("serialize_tuple_variant_elt"),
        fields,
        generics,
        true,
    );

    let value_expr = builder.expr().tuple()
        .with_exprs(
            field_names.iter().map(|field| {
                builder.expr().id(field)
            })
        )
        .build();

    quote_expr!(cx, {
        $visitor_struct
        $visitor_impl
        _serializer.serialize_tuple_variant($type_name, $variant_index, $variant_name, Visitor {
            value: $value_expr,
            state: 0,
            _structure_ty: ::std::marker::PhantomData::<&$structure_ty>,
        })
    })
}

fn serialize_struct_variant(
    cx: &ExtCtxt,
    builder: &aster::AstBuilder,
    variant_index: usize,
    variant_name: P<ast::Expr>,
    generics: &ast::Generics,
    ty: P<ast::Ty>,
    fields: &[item::Field],
    field_names: Vec<Ident>,
    container_attrs: &attr::ContainerAttrs,
) -> P<ast::Expr> {
    let variant_generics = builder.generics()
        .with(generics.clone())
        .add_lifetime_bound("'__serde_variant")
        .lifetime_name("'__serde_variant")
        .build();

    let variant_struct = builder.item().struct_("__VariantStruct")
        .with_generics(variant_generics.clone())
        .with_fields(
            fields.iter().map(|field| {
                builder.struct_field(field.ident.expect("struct has unnamed fields"))
                    .ty()
                    .ref_()
                    .lifetime("'__serde_variant")
                    .build_ty(field.ty.clone())
            })
        )
        .field("__serde_container_ty")
            .ty().phantom_data().build(ty.clone())
        .build();

    let variant_expr = builder.expr().struct_id("__VariantStruct")
        .with_id_exprs(
            fields.iter()
                .zip(field_names.iter())
                .map(|(field, field_name)| {
                    (
                        field.ident.expect("struct has unnamed fields"),
                        builder.expr().id(field_name),
                    )
                })
        )
        .field("__serde_container_ty").path()
            .global()
            .id("std").id("marker")
            .segment("PhantomData")
                .with_ty(ty.clone())
                .build()
            .build()
        .build();

    let variant_ty = builder.ty().path()
        .segment("__VariantStruct")
            .with_generics(variant_generics.clone())
            .build()
        .build();

    let (visitor_struct, visitor_impl) = serialize_struct_visitor(
        cx,
        builder,
        variant_ty.clone(),
        variant_ty.clone(),
        builder.id("serialize_struct_variant_elt"),
        fields,
        &variant_generics,
        true,
    );

    let container_name = container_attrs.name().serialize_name_expr();

    quote_expr!(cx, {
        $variant_struct
        $visitor_struct
        $visitor_impl
        _serializer.serialize_struct_variant(
            $container_name,
            $variant_index,
            $variant_name,
            Visitor {
                value: $variant_expr,
                state: 0,
                _structure_ty: ::std::marker::PhantomData,
            },
        )
    })
}

fn serialize_tuple_struct_visitor(
    cx: &ExtCtxt,
    builder: &aster::AstBuilder,
    structure_ty: P<ast::Ty>,
    variant_ty: P<ast::Ty>,
    serializer_method: ast::Ident,
    fields: &[item::Field],
    generics: &ast::Generics,
    is_enum: bool,
) -> (P<ast::Item>, P<ast::Item>) {
    let arms: Vec<_> = fields.iter()
        .enumerate()
        .map(|(i, field)| {
            let mut field_expr = builder.expr().tup_field(i).field("value").self_();
            if !is_enum {
                field_expr = quote_expr!(cx, &$field_expr);
            }

            let continue_if_skip = field.attrs.skip_serializing_if()
                .map(|path| quote_stmt!(cx, if $path($field_expr) { continue }));

            if let Some(path) = field.attrs.serialize_with() {
                field_expr = wrap_serialize_with(cx, builder,
                    &structure_ty, generics, &field.ty, path, field_expr);
            }

            quote_arm!(cx,
                $i => {
                    self.state += 1;
                    $continue_if_skip
                    Ok(Some(try!(_serializer.$serializer_method($field_expr))))
                }
            )
        })
        .collect();

    let visitor_impl_generics = builder.from_generics(generics.clone())
        .add_lifetime_bound("'__a")
        .lifetime_name("'__a")
        .build();

    let where_clause = &visitor_impl_generics.where_clause;

    let visitor_generics = builder.from_generics(visitor_impl_generics.clone())
        .strip_bounds()
        .build();

    let nfields = fields.len();

    (
        quote_item!(cx,
            struct Visitor $visitor_impl_generics $where_clause {
                state: usize,
                value: $variant_ty,
                _structure_ty: ::std::marker::PhantomData<&'__a $structure_ty>,
            }
        ).unwrap(),

        quote_item!(cx,
            impl $visitor_impl_generics _serde::ser::SeqVisitor
            for Visitor $visitor_generics
            $where_clause {
                #[inline]
                fn visit<__S>(&mut self, _serializer: &mut __S) -> ::std::result::Result<Option<()>, __S::Error>
                    where __S: _serde::ser::Serializer
                {
                    match self.state {
                        $arms
                        _ => Ok(None)
                    }
                }

                #[inline]
                fn len(&self) -> Option<usize> {
                    Some($nfields)
                }
            }
        ).unwrap(),
    )
}

fn serialize_struct_visitor(
    cx: &ExtCtxt,
    builder: &aster::AstBuilder,
    structure_ty: P<ast::Ty>,
    variant_ty: P<ast::Ty>,
    serializer_method: ast::Ident,
    fields: &[item::Field],
    generics: &ast::Generics,
    is_enum: bool,
) -> (P<ast::Item>, P<ast::Item>) {
    let arms: Vec<ast::Arm> = fields.iter()
        .filter(|&field| !field.attrs.skip_serializing())
        .enumerate()
        .map(|(i, field)| {
            let ident = field.ident.expect("struct has unnamed field");
            let mut field_expr = quote_expr!(cx, self.value.$ident);
            if !is_enum {
                field_expr = quote_expr!(cx, &$field_expr);
            }

            let key_expr = field.attrs.name().serialize_name_expr();

            let continue_if_skip = field.attrs.skip_serializing_if()
                .map(|path| quote_stmt!(cx, if $path($field_expr) { continue }));

            if let Some(path) = field.attrs.serialize_with() {
                field_expr = wrap_serialize_with(cx, builder,
                    &structure_ty, generics, &field.ty, path, field_expr)
            }

            let expr = quote_expr!(cx,
                _serializer.$serializer_method($key_expr, $field_expr)
            );

            quote_arm!(cx,
                $i => {
                    self.state += 1;
                    $continue_if_skip
                    return Ok(Some(try!($expr)));
                }
            )
        })
        .collect();

    let visitor_impl_generics = builder.from_generics(generics.clone())
        .add_lifetime_bound("'__a")
        .lifetime_name("'__a")
        .build();

    let where_clause = &visitor_impl_generics.where_clause;

    let visitor_generics = builder.from_generics(visitor_impl_generics.clone())
        .strip_bounds()
        .build();

    let len = fields.iter()
        .filter(|&field| !field.attrs.skip_serializing())
        .map(|field| {
            let ident = field.ident.expect("struct has unnamed fields");
            let mut field_expr = quote_expr!(cx, self.value.$ident);
            if !is_enum {
                field_expr = quote_expr!(cx, &$field_expr);
            }

            match field.attrs.skip_serializing_if() {
                Some(path) => quote_expr!(cx, if $path($field_expr) { 0 } else { 1 }),
                None => quote_expr!(cx, 1),
            }
        })
        .reduce(|sum, expr| quote_expr!(cx, $sum + $expr))
        .unwrap_or(quote_expr!(cx, 0));

    (
        quote_item!(cx,
            struct Visitor $visitor_impl_generics $where_clause {
                state: usize,
                value: $variant_ty,
                _structure_ty: ::std::marker::PhantomData<&'__a $structure_ty>,
            }
        ).unwrap(),

        quote_item!(cx,
            impl $visitor_impl_generics
            _serde::ser::MapVisitor
            for Visitor $visitor_generics
            $where_clause {
                #[inline]
                fn visit<__S>(&mut self, _serializer: &mut __S) -> ::std::result::Result<Option<()>, __S::Error>
                    where __S: _serde::ser::Serializer,
                {
                    loop {
                        match self.state {
                            $arms
                            _ => { return Ok(None); }
                        }
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

fn wrap_serialize_with(
    cx: &ExtCtxt,
    builder: &aster::AstBuilder,
    container_ty: &P<ast::Ty>,
    generics: &ast::Generics,
    field_ty: &P<ast::Ty>,
    path: &ast::Path,
    value: P<ast::Expr>,
) -> P<ast::Expr> {
    let where_clause = &generics.where_clause;

    let wrapper_generics = builder.from_generics(generics.clone())
        .add_lifetime_bound("'__a")
        .lifetime_name("'__a")
        .build();

    let wrapper_ty = builder.path()
        .segment("__SerializeWith")
            .with_generics(wrapper_generics.clone())
            .build()
        .build();

    quote_expr!(cx, {
        struct __SerializeWith $wrapper_generics $where_clause {
            value: &'__a $field_ty,
            phantom: ::std::marker::PhantomData<$container_ty>,
        }

        impl $wrapper_generics _serde::ser::Serialize for $wrapper_ty $where_clause {
            fn serialize<__S>(&self, __s: &mut __S) -> Result<(), __S::Error>
                where __S: _serde::ser::Serializer
            {
                $path(self.value, __s)
            }
        }

        __SerializeWith {
            value: $value,
            phantom: ::std::marker::PhantomData::<$container_ty>,
        }
    })
}
