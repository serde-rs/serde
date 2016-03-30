use aster;

use syntax::ast::{
    Ident,
    MetaItem,
    Item,
};
use syntax::ast;
use syntax::codemap::Span;
use syntax::ext::base::{Annotatable, ExtCtxt};
use syntax::ext::build::AstBuilder;
use syntax::ptr::P;

use attr;
use error::Error;

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
    item: &Item,
) -> Result<P<ast::Item>, Error> {
    let generics = match item.node {
        ast::ItemKind::Struct(_, ref generics) => generics,
        ast::ItemKind::Enum(_, ref generics) => generics,
        _ => {
            cx.span_err(
                item.span,
                "`#[derive(Serialize)]` may only be applied to structs and enums");
            return Err(Error);
        }
    };

    let impl_generics = builder.from_generics(generics.clone())
        .add_ty_param_bound(
            builder.path().global().ids(&["serde", "ser", "Serialize"]).build()
        )
        .build();

    let ty = builder.ty().path()
        .segment(item.ident).with_generics(impl_generics.clone()).build()
        .build();

    let body = try!(serialize_body(cx,
                                   &builder,
                                   &item,
                                   &impl_generics,
                                   ty.clone()));

    let where_clause = &impl_generics.where_clause;

    Ok(quote_item!(cx,
        impl $impl_generics ::serde::ser::Serialize for $ty $where_clause {
            fn serialize<__S>(&self, _serializer: &mut __S) -> ::std::result::Result<(), __S::Error>
                where __S: ::serde::ser::Serializer,
            {
                $body
            }
        }
    ).unwrap())
}

fn serialize_body(
    cx: &ExtCtxt,
    builder: &aster::AstBuilder,
    item: &Item,
    impl_generics: &ast::Generics,
    ty: P<ast::Ty>,
) -> Result<P<ast::Expr>, Error> {
    let container_attrs = try!(attr::ContainerAttrs::from_item(cx, item));

    match item.node {
        ast::ItemKind::Struct(ref variant_data, _) => {
            serialize_item_struct(
                cx,
                builder,
                impl_generics,
                ty,
                item.span,
                variant_data,
                &container_attrs,
            )
        }
        ast::ItemKind::Enum(ref enum_def, _) => {
            serialize_item_enum(
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
                        "expected ItemStruct or ItemEnum in #[derive(Serialize)]");
        }
    }
}

fn serialize_item_struct(
    cx: &ExtCtxt,
    builder: &aster::AstBuilder,
    impl_generics: &ast::Generics,
    ty: P<ast::Ty>,
    span: Span,
    variant_data: &ast::VariantData,
    container_attrs: &attr::ContainerAttrs,
) -> Result<P<ast::Expr>, Error> {
    match *variant_data {
        ast::VariantData::Unit(_) => {
            serialize_unit_struct(
                cx,
                container_attrs,
            )
        }
        ast::VariantData::Tuple(ref fields, _) if fields.len() == 1 => {
            serialize_newtype_struct(
                cx,
                container_attrs,
            )
        }
        ast::VariantData::Tuple(ref fields, _) => {
            if fields.iter().any(|field| !field.node.kind.is_unnamed()) {
                cx.span_bug(span, "tuple struct has named fields")
            }

            serialize_tuple_struct(
                cx,
                &builder,
                impl_generics,
                ty,
                fields.len(),
                container_attrs,
            )
        }
        ast::VariantData::Struct(ref fields, _) => {
            if fields.iter().any(|field| field.node.kind.is_unnamed()) {
                cx.span_bug(span, "struct has unnamed fields")
            }

            serialize_struct(
                cx,
                &builder,
                impl_generics,
                ty,
                fields,
                container_attrs,
            )
        }
    }
}

fn serialize_unit_struct(
    cx: &ExtCtxt,
    container_attrs: &attr::ContainerAttrs,
) -> Result<P<ast::Expr>, Error> {
    let type_name = container_attrs.name().serialize_name_expr();

    Ok(quote_expr!(cx,
        _serializer.serialize_unit_struct($type_name)
    ))
}

fn serialize_newtype_struct(
    cx: &ExtCtxt,
    container_attrs: &attr::ContainerAttrs,
) -> Result<P<ast::Expr>, Error> {
    let type_name = container_attrs.name().serialize_name_expr();

    Ok(quote_expr!(cx,
        _serializer.serialize_newtype_struct($type_name, &self.0)
    ))
}

fn serialize_tuple_struct(
    cx: &ExtCtxt,
    builder: &aster::AstBuilder,
    impl_generics: &ast::Generics,
    ty: P<ast::Ty>,
    fields: usize,
    container_attrs: &attr::ContainerAttrs,
) -> Result<P<ast::Expr>, Error> {
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
    );

    let type_name = container_attrs.name().serialize_name_expr();

    Ok(quote_expr!(cx, {
        $visitor_struct
        $visitor_impl
        _serializer.serialize_tuple_struct($type_name, Visitor {
            value: self,
            state: 0,
            _structure_ty: ::std::marker::PhantomData::<&$ty>,
        })
    }))
}

fn serialize_struct(
    cx: &ExtCtxt,
    builder: &aster::AstBuilder,
    impl_generics: &ast::Generics,
    ty: P<ast::Ty>,
    fields: &[ast::StructField],
    container_attrs: &attr::ContainerAttrs,
) -> Result<P<ast::Expr>, Error> {
    let (visitor_struct, visitor_impl) = try!(serialize_struct_visitor(
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
    ));

    let type_name = container_attrs.name().serialize_name_expr();

    Ok(quote_expr!(cx, {
        $visitor_struct
        $visitor_impl
        _serializer.serialize_struct($type_name, Visitor {
            value: self,
            state: 0,
            _structure_ty: ::std::marker::PhantomData::<&$ty>,
        })
    }))
}

fn serialize_item_enum(
    cx: &ExtCtxt,
    builder: &aster::AstBuilder,
    type_ident: Ident,
    impl_generics: &ast::Generics,
    ty: P<ast::Ty>,
    enum_def: &ast::EnumDef,
    container_attrs: &attr::ContainerAttrs,
) -> Result<P<ast::Expr>, Error> {
    let arms: Vec<_> = try!(
        enum_def.variants.iter()
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
            .collect()
    );

    Ok(quote_expr!(cx,
        match *self {
            $arms
        }
    ))
}

fn serialize_variant(
    cx: &ExtCtxt,
    builder: &aster::AstBuilder,
    type_ident: Ident,
    generics: &ast::Generics,
    ty: P<ast::Ty>,
    variant: &ast::Variant,
    variant_index: usize,
    container_attrs: &attr::ContainerAttrs,
) -> Result<ast::Arm, Error> {
    let type_name = container_attrs.name().serialize_name_expr();

    let variant_ident = variant.node.name;
    let variant_attrs = try!(attr::VariantAttrs::from_variant(cx, variant));
    let variant_name = variant_attrs.name().serialize_name_expr();

    match variant.node.data {
        ast::VariantData::Unit(_) => {
            let pat = builder.pat().path()
                .id(type_ident).id(variant_ident)
                .build();

            Ok(quote_arm!(cx,
                $pat => {
                    ::serde::ser::Serializer::serialize_unit_variant(
                        _serializer,
                        $type_name,
                        $variant_index,
                        $variant_name,
                    )
                }
            ))
        },
        ast::VariantData::Tuple(ref fields, _) if fields.len() == 1 => {
            let field = builder.id("__simple_value");
            let field = builder.pat().ref_id(field);
            let pat = builder.pat().enum_()
                .id(type_ident).id(variant_ident).build()
                .with_pats(Some(field).into_iter())
                .build();

            Ok(quote_arm!(cx,
                $pat => {
                    ::serde::ser::Serializer::serialize_newtype_variant(
                        _serializer,
                        $type_name,
                        $variant_index,
                        $variant_name,
                        __simple_value,
                    )
                }
            ))
        },
        ast::VariantData::Tuple(ref fields, _) => {
            let field_names: Vec<ast::Ident> = (0 .. fields.len())
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
                fields,
                field_names,
            );

            Ok(quote_arm!(cx,
                $pat => { $expr }
            ))
        }
        ast::VariantData::Struct(ref fields, _) => {
            let field_names: Vec<_> = (0 .. fields.len())
                .map(|i| builder.id(format!("__field{}", i)))
                .collect();

            let pat = builder.pat().struct_()
                .id(type_ident).id(variant_ident).build()
                .with_pats(
                    field_names.iter()
                        .zip(fields.iter())
                        .map(|(id, field)| {
                            let name = match field.node.kind {
                                ast::NamedField(name, _) => name,
                                ast::UnnamedField(_) => {
                                    cx.span_bug(field.span, "struct variant has unnamed fields")
                                }
                            };

                            (name, builder.pat().ref_id(id))
                        })
                )
                .build();

            let expr = try!(serialize_struct_variant(
                cx,
                builder,
                variant_index,
                variant_name,
                generics,
                ty,
                fields,
                field_names,
                container_attrs,
            ));

            Ok(quote_arm!(cx,
                $pat => { $expr }
            ))
        }
    }
}

fn serialize_tuple_variant(
    cx: &ExtCtxt,
    builder: &aster::AstBuilder,
    type_name: P<ast::Expr>,
    variant_index: usize,
    variant_name: P<ast::Expr>,
    generics: &ast::Generics,
    structure_ty: P<ast::Ty>,
    fields: &[ast::StructField],
    field_names: Vec<Ident>,
) -> P<ast::Expr> {
    let variant_ty = builder.ty().tuple()
        .with_tys(
            fields.iter().map(|field| {
                builder.ty()
                    .ref_()
                    .lifetime("'__a")
                    .build_ty(field.node.ty.clone())
            })
        )
        .build();

    let (visitor_struct, visitor_impl) = serialize_tuple_struct_visitor(
        cx,
        builder,
        structure_ty.clone(),
        variant_ty,
        builder.id("serialize_tuple_variant_elt"),
        fields.len(),
        generics,
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
    fields: &[ast::StructField],
    field_names: Vec<Ident>,
    container_attrs: &attr::ContainerAttrs,
) -> Result<P<ast::Expr>, Error> {
    let variant_generics = builder.generics()
        .with(generics.clone())
        .add_lifetime_bound("'__serde_variant")
        .lifetime_name("'__serde_variant")
        .build();

    let variant_struct = builder.item().struct_("__VariantStruct")
        .with_generics(variant_generics.clone())
        .with_fields(
            fields.iter().map(|field| {
                builder.struct_field(field.node.ident().expect("struct has unnamed fields"))
                    .with_attrs(field.node.attrs.iter().cloned())
                    .ty()
                    .ref_()
                    .lifetime("'__serde_variant")
                    .build_ty(field.node.ty.clone())
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
                        field.node.ident().expect("struct has unnamed fields"),
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

    let (visitor_struct, visitor_impl) = try!(serialize_struct_visitor(
        cx,
        builder,
        variant_ty.clone(),
        variant_ty.clone(),
        builder.id("serialize_struct_variant_elt"),
        fields,
        &variant_generics,
        true,
    ));

    let container_name = container_attrs.name().serialize_name_expr();

    Ok(quote_expr!(cx, {
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
    }))
}

fn serialize_tuple_struct_visitor(
    cx: &ExtCtxt,
    builder: &aster::AstBuilder,
    structure_ty: P<ast::Ty>,
    variant_ty: P<ast::Ty>,
    serializer_method: ast::Ident,
    fields: usize,
    generics: &ast::Generics
) -> (P<ast::Item>, P<ast::Item>) {
    let arms: Vec<ast::Arm> = (0 .. fields)
        .map(|i| {
            let expr = builder.expr().method_call(serializer_method)
                .id("_serializer")
                .arg().ref_().tup_field(i).field("value").self_()
                .build();

            quote_arm!(cx,
                $i => {
                    self.state += 1;
                    Ok(Some(try!($expr)))
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

    (
        quote_item!(cx,
            struct Visitor $visitor_impl_generics $where_clause {
                state: usize,
                value: $variant_ty,
                _structure_ty: ::std::marker::PhantomData<&'__a $structure_ty>,
            }
        ).unwrap(),

        quote_item!(cx,
            impl $visitor_impl_generics ::serde::ser::SeqVisitor
            for Visitor $visitor_generics
            $where_clause {
                #[inline]
                fn visit<S>(&mut self, _serializer: &mut S) -> ::std::result::Result<Option<()>, S::Error>
                    where S: ::serde::ser::Serializer
                {
                    match self.state {
                        $arms
                        _ => Ok(None)
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

fn serialize_struct_visitor(
    cx: &ExtCtxt,
    builder: &aster::AstBuilder,
    structure_ty: P<ast::Ty>,
    variant_ty: P<ast::Ty>,
    serializer_method: ast::Ident,
    fields: &[ast::StructField],
    generics: &ast::Generics,
    is_enum: bool,
) -> Result<(P<ast::Item>, P<ast::Item>), Error> {
    let field_attrs = try!(
        attr::get_struct_field_attrs(cx, &structure_ty, generics, fields, is_enum)
    );

    let arms: Vec<ast::Arm> = fields.iter().zip(field_attrs.iter())
        .filter(|&(_, ref field_attr)| !field_attr.skip_serializing_field())
        .enumerate()
        .map(|(i, (ref field, ref field_attr))| {
            let name = field.node.ident().expect("struct has unnamed field");

            let key_expr = field_attr.name().serialize_name_expr();

            let stmt = if let Some(expr) = field_attr.skip_serializing_field_if() {
                    Some(quote_stmt!(cx, if $expr { continue; }))
            } else {
                None
            };

            let field_expr = match field_attr.serialize_with() {
                Some(expr) => expr.clone(),
                None => quote_expr!(cx, &self.value.$name),
            };

            let expr = quote_expr!(cx,
                _serializer.$serializer_method($key_expr, $field_expr)
            );

            quote_arm!(cx,
                $i => {
                    self.state += 1;
                    $stmt
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

    let len = field_attrs.iter()
        .filter(|field_attr| !field_attr.skip_serializing_field())
        .map(|field_attr| {
            match field_attr.skip_serializing_field_if() {
                Some(expr) => {
                    quote_expr!(cx, if $expr { 0 } else { 1 })
                }
                None => {
                    quote_expr!(cx, 1)
                }
            }
        })
        .fold(quote_expr!(cx, 0), |sum, expr| quote_expr!(cx, $sum + $expr));

    Ok((
        quote_item!(cx,
            struct Visitor $visitor_impl_generics $where_clause {
                state: usize,
                value: $variant_ty,
                _structure_ty: ::std::marker::PhantomData<&'__a $structure_ty>,
            }
        ).unwrap(),

        quote_item!(cx,
            impl $visitor_impl_generics
            ::serde::ser::MapVisitor
            for Visitor $visitor_generics
            $where_clause {
                #[inline]
                fn visit<S>(&mut self, _serializer: &mut S) -> ::std::result::Result<Option<()>, S::Error>
                    where S: ::serde::ser::Serializer,
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
    ))
}
