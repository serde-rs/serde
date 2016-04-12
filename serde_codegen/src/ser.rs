use std::collections::HashSet;

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
use syntax::visit;

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

    let impl_generics = build_impl_generics(cx, builder, item, generics);

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

// All the generics in the input, plus a bound `T: Serialize` for each generic
// field type that will be serialized by us.
fn build_impl_generics(
    cx: &ExtCtxt,
    builder: &aster::AstBuilder,
    item: &Item,
    generics: &ast::Generics,
) -> ast::Generics {
    let serialize_path = builder.path()
        .global()
        .ids(&["serde", "ser", "Serialize"])
        .build();

    builder.from_generics(generics.clone())
        .with_predicates(
            all_variants(cx, item).iter()
                .flat_map(|variant_data| all_struct_fields(variant_data))
                .filter(|field| serialized_by_us(field))
                .map(|field| &field.ty)
                // TODO this filter can be removed later, see comment on function
                .filter(|ty| contains_generic(ty, generics))
                .map(|ty| strip_reference(ty))
                .map(|ty| builder.where_predicate()
                    // the type that is being bounded i.e. T
                    .bound().build(ty.clone())
                    // the bound i.e. Serialize
                    .bound().trait_(serialize_path.clone()).build()
                    .build()))
        .build()
}

fn all_variants<'a>(cx: &ExtCtxt, item: &'a Item) -> Vec<&'a ast::VariantData> {
    match item.node {
        ast::ItemKind::Struct(ref variant_data, _) => {
            vec![variant_data]
        }
        ast::ItemKind::Enum(ref enum_def, _) => {
            enum_def.variants.iter()
                .map(|variant| &variant.node.data)
                .collect()
        }
        _ => {
            cx.span_bug(item.span,
                        "expected Item to be Struct or Enum in #[derive(Serialize)]");
        }
    }
}

fn all_struct_fields(variant_data: &ast::VariantData) -> &[ast::StructField] {
    match *variant_data {
        ast::VariantData::Struct(ref fields, _) |
        ast::VariantData::Tuple(ref fields, _) => {
            fields
        }
        ast::VariantData::Unit(_) => {
            &[]
        }
    }
}

// Fields with a `skip_serializing` or `serialize_with` attribute are not
// serialized by us. All other fields may need a `T: Serialize` bound where T is
// the type of the field.
fn serialized_by_us(field: &ast::StructField) -> bool {
    for meta_items in field.attrs.iter().filter_map(attr::get_serde_meta_items) {
        for meta_item in meta_items {
            match meta_item.node {
                ast::MetaItemKind::Word(ref name) if name == &"skip_serializing" => {
                    return false
                }
                ast::MetaItemKind::NameValue(ref name, _) if name == &"serialize_with" => {
                    return false
                }
                _ => {}
            }
        }
    }
    true
}

// Rust <1.7 enforces that `where` clauses involve generic type parameters. The
// corresponding compiler error is E0193. It is no longer enforced in Rust >=1.7
// so this filtering can be removed in the future when we stop supporting <1.7.
//
// E0193 means we must not generate a `where` clause like `i32: Serialize`
// because even though i32 implements Serialize, i32 is not a generic type
// parameter. Clauses like `T: Serialize` and `Option<T>: Serialize` are okay.
// This function decides whether a given type references any of the generic type
// parameters in the input `Generics`.
fn contains_generic(ty: &ast::Ty, generics: &ast::Generics) -> bool {
    struct FindGeneric<'a> {
        generic_names: &'a HashSet<ast::Name>,
        found_generic: bool,
    }
    impl<'a, 'v> visit::Visitor<'v> for FindGeneric<'a> {
        fn visit_path(&mut self, path: &'v ast::Path, _id: ast::NodeId) {
            if !path.global
                    && path.segments.len() == 1
                    && self.generic_names.contains(&path.segments[0].identifier.name) {
                self.found_generic = true;
            } else {
                visit::walk_path(self, path);
            }
        }
    }

    let generic_names: HashSet<_> = generics.ty_params.iter()
        .map(|ty_param| ty_param.ident.name)
        .collect();

    let mut visitor = FindGeneric {
        generic_names: &generic_names,
        found_generic: false,
    };
    visit::walk_ty(&mut visitor, ty);
    visitor.found_generic
}

// This is required to handle types that use both a reference and a value of
// the same type, as in:
//
//    enum Test<'a, T> where T: 'a {
//        Lifetime(&'a T),
//        NoLifetime(T),
//    }
//
// Preserving references, we would generate an impl like:
//
//    impl<'a, T> Serialize for Test<'a, T>
//        where &'a T: Serialize,
//              T: Serialize { ... }
//
// And taking a reference to one of the elements would fail with:
//
//    error: cannot infer an appropriate lifetime for pattern due
//    to conflicting requirements [E0495]
//        Test::NoLifetime(ref v) => { ... }
//                         ^~~~~
//
// Instead, we strip references before adding `T: Serialize` bounds in order to
// generate:
//
//    impl<'a, T> Serialize for Test<'a, T>
//        where T: Serialize { ... }
fn strip_reference(ty: &P<ast::Ty>) -> &P<ast::Ty> {
    match ty.node {
        ast::TyKind::Rptr(_, ref mut_ty) => &mut_ty.ty,
        _ => ty
    }
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
            if fields.iter().any(|field| field.ident.is_some()) {
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
            let name = field.ident.expect("struct has unnamed field");

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
