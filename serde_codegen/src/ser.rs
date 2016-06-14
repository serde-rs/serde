use aster;

use syntax::ast::{
    Ident,
    MetaItem,
    Item,
};
use syntax::ast;
use syntax::codemap::Span;
use syntax::ext::base::{Annotatable, ExtCtxt};
use syntax::ptr::P;

use attr;
use bound;
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

    let container_attrs = try!(attr::ContainerAttrs::from_item(cx, item));

    let impl_generics = try!(build_impl_generics(cx, builder, item, generics, &container_attrs));

    let ty = builder.ty().path()
        .segment(item.ident).with_generics(impl_generics.clone()).build()
        .build();

    let body = try!(serialize_body(cx,
                                   &builder,
                                   &item,
                                   &impl_generics,
                                   ty.clone(),
                                   &container_attrs));

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
    cx: &ExtCtxt,
    builder: &aster::AstBuilder,
    item: &Item,
    generics: &ast::Generics,
    container_attrs: &attr::ContainerAttrs,
) -> Result<ast::Generics, Error> {
    let generics = bound::without_defaults(generics);

    let generics = try!(bound::with_where_predicates_from_fields(
        cx, builder, item, &generics,
        |attrs| attrs.ser_bound()));

    match container_attrs.ser_bound() {
        Some(predicates) => {
            let generics = bound::with_where_predicates(builder, &generics, predicates);
            Ok(generics)
        }
        None => {
            let generics = try!(bound::with_bound(cx, builder, item, &generics,
                needs_serialize_bound,
                &builder.path().ids(&["_serde", "ser", "Serialize"]).build()));
            Ok(generics)
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
    item: &Item,
    impl_generics: &ast::Generics,
    ty: P<ast::Ty>,
    container_attrs: &attr::ContainerAttrs,
) -> Result<P<ast::Expr>, Error> {
    match item.node {
        ast::ItemKind::Struct(ref variant_data, _) => {
            serialize_item_struct(
                cx,
                builder,
                impl_generics,
                ty,
                item.span,
                variant_data,
                container_attrs,
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
                container_attrs,
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
                &builder,
                impl_generics,
                ty,
                &fields[0],
                container_attrs,
            )
        }
        ast::VariantData::Tuple(ref fields, _) => {
            if fields.iter().any(|field| field.ident.is_some()) {
                cx.span_bug(span, "tuple struct has named fields")
            }

            serialize_tuple_struct(
                cx,
                &builder,
                impl_generics,
                ty,
                fields,
                container_attrs,
            )
        }
        ast::VariantData::Struct(ref fields, _) => {
            if fields.iter().any(|field| field.ident.is_none()) {
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
    builder: &aster::AstBuilder,
    impl_generics: &ast::Generics,
    container_ty: P<ast::Ty>,
    field: &ast::StructField,
    container_attrs: &attr::ContainerAttrs,
) -> Result<P<ast::Expr>, Error> {
    let type_name = container_attrs.name().serialize_name_expr();

    let attrs = try!(attr::FieldAttrs::from_field(cx, 0, field));

    let mut field_expr = quote_expr!(cx, &self.0);
    if let Some(path) = attrs.serialize_with() {
        field_expr = wrap_serialize_with(cx, builder,
            &container_ty, impl_generics, &field.ty, path, field_expr);
    }

    Ok(quote_expr!(cx,
        _serializer.serialize_newtype_struct($type_name, $field_expr)
    ))
}

fn serialize_tuple_struct(
    cx: &ExtCtxt,
    builder: &aster::AstBuilder,
    impl_generics: &ast::Generics,
    ty: P<ast::Ty>,
    fields: &[ast::StructField],
    container_attrs: &attr::ContainerAttrs,
) -> Result<P<ast::Expr>, Error> {
    let (visitor_struct, visitor_impl) = try!(serialize_tuple_struct_visitor(
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
    ));

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
            Ok(quote_arm!(cx,
                $type_ident::$variant_ident => {
                    _serde::ser::Serializer::serialize_unit_variant(
                        _serializer,
                        $type_name,
                        $variant_index,
                        $variant_name,
                    )
                }
            ))
        },
        ast::VariantData::Tuple(ref fields, _) if fields.len() == 1 => {
            let expr = try!(serialize_newtype_variant(
                cx,
                builder,
                type_name,
                variant_index,
                variant_name,
                ty,
                generics,
                &fields[0],
            ));

            Ok(quote_arm!(cx,
                $type_ident::$variant_ident(ref __simple_value) => { $expr }
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

            let expr = try!(serialize_tuple_variant(
                cx,
                builder,
                type_name,
                variant_index,
                variant_name,
                generics,
                ty,
                fields,
                field_names,
            ));

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

fn serialize_newtype_variant(
    cx: &ExtCtxt,
    builder: &aster::AstBuilder,
    type_name: P<ast::Expr>,
    variant_index: usize,
    variant_name: P<ast::Expr>,
    container_ty: P<ast::Ty>,
    generics: &ast::Generics,
    field: &ast::StructField,
) -> Result<P<ast::Expr>, Error> {
    let attrs = try!(attr::FieldAttrs::from_field(cx, 0, field));

    let mut field_expr = quote_expr!(cx, __simple_value);
    if let Some(path) = attrs.serialize_with() {
        field_expr = wrap_serialize_with(cx, builder,
            &container_ty, generics, &field.ty, path, field_expr);
    }

    Ok(quote_expr!(cx,
        _serde::ser::Serializer::serialize_newtype_variant(
            _serializer,
            $type_name,
            $variant_index,
            $variant_name,
            $field_expr,
        )
    ))
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
) -> Result<P<ast::Expr>, Error> {
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

    let (visitor_struct, visitor_impl) = try!(serialize_tuple_struct_visitor(
        cx,
        builder,
        structure_ty.clone(),
        variant_ty,
        builder.id("serialize_tuple_variant_elt"),
        fields,
        generics,
        true,
    ));

    let value_expr = builder.expr().tuple()
        .with_exprs(
            field_names.iter().map(|field| {
                builder.expr().id(field)
            })
        )
        .build();

    Ok(quote_expr!(cx, {
        $visitor_struct
        $visitor_impl
        _serializer.serialize_tuple_variant($type_name, $variant_index, $variant_name, Visitor {
            value: $value_expr,
            state: 0,
            _structure_ty: ::std::marker::PhantomData::<&$structure_ty>,
        })
    }))
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
                builder.struct_field(field.ident.expect("struct has unnamed fields"))
                    .with_attrs(field.attrs.iter().cloned())
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
    fields: &[ast::StructField],
    generics: &ast::Generics,
    is_enum: bool,
) -> Result<(P<ast::Item>, P<ast::Item>), Error> {
    let fields_with_attrs = try!(attr::fields_with_attrs(cx, fields));

    let arms: Vec<_> = fields_with_attrs.iter()
        .enumerate()
        .map(|(i, &(ref field, ref attrs))| {
            let mut field_expr = builder.expr().tup_field(i).field("value").self_();
            if !is_enum {
                field_expr = quote_expr!(cx, &$field_expr);
            }

            let continue_if_skip = attrs.skip_serializing_if()
                .map(|path| quote_stmt!(cx, if $path($field_expr) { continue }));

            if let Some(path) = attrs.serialize_with() {
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

    Ok((
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
    ))
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
    let fields_with_attrs = try!(attr::fields_with_attrs(cx, fields));

    let arms: Vec<ast::Arm> = fields_with_attrs.iter()
        .filter(|&&(_, ref attrs)| !attrs.skip_serializing())
        .enumerate()
        .map(|(i, &(ref field, ref attrs))| {
            let ident = field.ident.expect("struct has unnamed field");
            let mut field_expr = quote_expr!(cx, self.value.$ident);
            if !is_enum {
                field_expr = quote_expr!(cx, &$field_expr);
            }

            let key_expr = attrs.name().serialize_name_expr();

            let continue_if_skip = attrs.skip_serializing_if()
                .map(|path| quote_stmt!(cx, if $path($field_expr) { continue }));

            if let Some(path) = attrs.serialize_with() {
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

    let len = fields_with_attrs.iter()
        .filter(|&&(_, ref attrs)| !attrs.skip_serializing())
        .map(|&(ref field, ref attrs)| {
            let ident = field.ident.expect("struct has unnamed fields");
            let mut field_expr = quote_expr!(cx, self.value.$ident);
            if !is_enum {
                field_expr = quote_expr!(cx, &$field_expr);
            }

            match attrs.skip_serializing_if() {
                Some(path) => quote_expr!(cx, if $path($field_expr) { 0 } else { 1 }),
                None => quote_expr!(cx, 1),
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
    ))
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
