use aster;

use syntax::ast::{self, Ident, MetaItem};
use syntax::codemap::Span;
use syntax::ext::base::{Annotatable, ExtCtxt};
use syntax::ptr::P;

use bound;
use span;
use internals::ast::{Body, Field, Item, Style, Variant};
use internals::{attr, Error};

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

    let item = match Item::from_ast(cx, item) {
        Ok(item) => item,
        Err(Error::UnexpectedItemKind) => {
            cx.span_err(item.span,
                "`#[derive(Serialize)]` may only be applied to structs and enums");
            return;
        }
    };

    let builder = aster::AstBuilder::new().span(span);

    let impl_item = serialize_item(cx, &builder, &item);
    push(span::record_expansion(cx, impl_item, "Serialize"))
}

fn serialize_item(
    cx: &ExtCtxt,
    builder: &aster::AstBuilder,
    item: &Item,
) -> P<ast::Item> {
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

    quote_item!(cx,
        #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
        const $dummy_const: () = {
            extern crate serde as _serde;
            #[automatically_derived]
            impl $impl_generics _serde::ser::Serialize for $ty $where_clause {
                fn serialize<__S>(&self, _serializer: &mut __S) -> ::std::result::Result<(), __S::Error>
                    where __S: _serde::ser::Serializer
                $body
            }
        };
    ).unwrap()
}

// All the generics in the input, plus a bound `T: Serialize` for each generic
// field type that will be serialized by us.
fn build_impl_generics(
    builder: &aster::AstBuilder,
    item: &Item,
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
fn needs_serialize_bound(attrs: &attr::Field) -> bool {
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
) -> P<ast::Block> {
    match item.body {
        Body::Enum(ref variants) => {
            serialize_item_enum(
                cx,
                builder,
                item.ident,
                impl_generics,
                ty,
                variants,
                &item.attrs)
        }
        Body::Struct(Style::Struct, ref fields) => {
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
        Body::Struct(Style::Tuple, ref fields) => {
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
        Body::Struct(Style::Newtype, ref fields) => {
            serialize_newtype_struct(
                cx,
                builder,
                impl_generics,
                ty,
                &fields[0],
                &item.attrs)
        }
        Body::Struct(Style::Unit, _) => {
            serialize_unit_struct(
                cx,
                builder,
                &item.attrs)
        }
    }
}

fn serialize_unit_struct(
    cx: &ExtCtxt,
    builder: &aster::AstBuilder,
    item_attrs: &attr::Item,
) -> P<ast::Block> {
    let type_name = name_expr(builder, item_attrs.name());

    quote_block!(cx, {
        _serializer.serialize_unit_struct($type_name)
    }).unwrap()
}

fn serialize_newtype_struct(
    cx: &ExtCtxt,
    builder: &aster::AstBuilder,
    impl_generics: &ast::Generics,
    item_ty: P<ast::Ty>,
    field: &Field,
    item_attrs: &attr::Item,
) -> P<ast::Block> {
    let type_name = name_expr(builder, item_attrs.name());

    let mut field_expr = quote_expr!(cx, &self.0);
    if let Some(path) = field.attrs.serialize_with() {
        field_expr = wrap_serialize_with(cx, builder,
            &item_ty, impl_generics, &field.ty, path, field_expr);
    }

    quote_block!(cx, {
        _serializer.serialize_newtype_struct($type_name, $field_expr)
    }).unwrap()
}

fn serialize_tuple_struct(
    cx: &ExtCtxt,
    builder: &aster::AstBuilder,
    impl_generics: &ast::Generics,
    ty: P<ast::Ty>,
    fields: &[Field],
    item_attrs: &attr::Item,
) -> P<ast::Block> {
    let serialize_stmts = serialize_tuple_struct_visitor(
        cx,
        builder,
        ty.clone(),
        fields,
        impl_generics,
        false,
        cx.ident_of("serialize_tuple_struct_elt"),
    );

    let type_name = name_expr(builder, item_attrs.name());
    let len = serialize_stmts.len();

    quote_block!(cx, {
        let mut state = try!(_serializer.serialize_tuple_struct($type_name, $len));
        $serialize_stmts
        _serializer.serialize_tuple_struct_end(state)
    }).unwrap()
}

fn serialize_struct(
    cx: &ExtCtxt,
    builder: &aster::AstBuilder,
    impl_generics: &ast::Generics,
    ty: P<ast::Ty>,
    fields: &[Field],
    item_attrs: &attr::Item,
) -> P<ast::Block> {
    let serialize_fields = serialize_struct_visitor(
        cx,
        builder,
        ty.clone(),
        fields,
        impl_generics,
        false,
        cx.ident_of("serialize_struct_elt"),
    );

    let type_name = name_expr(builder, item_attrs.name());
    let len = fields.iter()
        .filter(|&field| !field.attrs.skip_serializing())
        .map(|field| {
            let ident = field.ident.expect("struct has unnamed fields");
            let field_expr = quote_expr!(cx, &self.$ident);

            match field.attrs.skip_serializing_if() {
                Some(path) => quote_expr!(cx, if $path($field_expr) { 0 } else { 1 }),
                None => quote_expr!(cx, 1),
            }
         })
        .fold(quote_expr!(cx, 0), |sum, expr| quote_expr!(cx, $sum + $expr));

    quote_block!(cx, {
        let mut state = try!(_serializer.serialize_struct($type_name, $len));
        $serialize_fields
        _serializer.serialize_struct_end(state)
    }).unwrap()
}

fn serialize_item_enum(
    cx: &ExtCtxt,
    builder: &aster::AstBuilder,
    type_ident: Ident,
    impl_generics: &ast::Generics,
    ty: P<ast::Ty>,
    variants: &[Variant],
    item_attrs: &attr::Item,
) -> P<ast::Block> {
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
                    item_attrs,
                )
            })
            .collect();

    quote_block!(cx, {
        match *self {
            $arms
        }
    }).unwrap()
}

fn serialize_variant(
    cx: &ExtCtxt,
    builder: &aster::AstBuilder,
    type_ident: Ident,
    generics: &ast::Generics,
    ty: P<ast::Ty>,
    variant: &Variant,
    variant_index: usize,
    item_attrs: &attr::Item,
) -> ast::Arm {
    let type_name = name_expr(builder, item_attrs.name());

    let variant_ident = variant.ident;
    let variant_name = name_expr(builder, variant.attrs.name());

    match variant.style {
        Style::Unit => {
            quote_arm!(cx,
                $type_ident::$variant_ident =>
                    _serde::ser::Serializer::serialize_unit_variant(
                        _serializer,
                        $type_name,
                        $variant_index,
                        $variant_name,
                    ),
            )
        },
        Style::Newtype => {
            let block = serialize_newtype_variant(
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
                $type_ident::$variant_ident(ref __simple_value) => $block
            )
        },
        Style::Tuple => {
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

            let block = serialize_tuple_variant(
                cx,
                builder,
                type_name,
                variant_index,
                variant_name,
                generics,
                ty,
                &variant.fields,
            );

            quote_arm!(cx,
                $pat => $block
            )
        }
        Style::Struct => {
            let mut pat = builder.pat().struct_().id(type_ident).id(variant_ident).build();
            for field in variant.fields.iter() {
                let name = match field.ident {
                    Some(name) => name,
                    None => cx.span_bug(field.span, "struct variant has unnamed fields"),
                };
                pat = pat.with_field_pat(ast::FieldPat {
                    ident: name,
                    pat: builder.pat().ref_id(name),
                    is_shorthand: true,
                });
            }
            let pat = pat.build();

            let block = serialize_struct_variant(
                cx,
                builder,
                variant_index,
                variant_name,
                generics,
                ty,
                &variant.fields,
                item_attrs,
            );

            quote_arm!(cx,
                $pat => $block
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
    item_ty: P<ast::Ty>,
    generics: &ast::Generics,
    field: &Field,
) -> P<ast::Block> {
    let mut field_expr = quote_expr!(cx, __simple_value);
    if let Some(path) = field.attrs.serialize_with() {
        field_expr = wrap_serialize_with(cx, builder,
            &item_ty, generics, &field.ty, path, field_expr);
    }

    quote_block!(cx, {
        _serde::ser::Serializer::serialize_newtype_variant(
            _serializer,
            $type_name,
            $variant_index,
            $variant_name,
            $field_expr,
        )
    }).unwrap()
}

fn serialize_tuple_variant(
    cx: &ExtCtxt,
    builder: &aster::AstBuilder,
    type_name: P<ast::Expr>,
    variant_index: usize,
    variant_name: P<ast::Expr>,
    generics: &ast::Generics,
    structure_ty: P<ast::Ty>,
    fields: &[Field],
) -> P<ast::Block> {
    let serialize_stmts = serialize_tuple_struct_visitor(
        cx,
        builder,
        structure_ty,
        fields,
        generics,
        true,
        cx.ident_of("serialize_tuple_variant_elt"),
    );

    let len = serialize_stmts.len();

    quote_block!(cx, {
        let mut state = try!(_serializer.serialize_tuple_variant($type_name, $variant_index, $variant_name, $len));
        $serialize_stmts
        _serializer.serialize_tuple_variant_end(state)
    }).unwrap()
}

fn serialize_struct_variant(
    cx: &ExtCtxt,
    builder: &aster::AstBuilder,
    variant_index: usize,
    variant_name: P<ast::Expr>,
    generics: &ast::Generics,
    ty: P<ast::Ty>,
    fields: &[Field],
    item_attrs: &attr::Item,
) -> P<ast::Block> {

    let serialize_fields = serialize_struct_visitor(
        cx,
        builder,
        ty.clone(),
        fields,
        &generics,
        true,
        cx.ident_of("serialize_struct_variant_elt"),
    );

    let item_name = name_expr(builder, item_attrs.name());
    let len = fields.iter()
        .filter(|&field| !field.attrs.skip_serializing())
        .map(|field| {
            let ident = field.ident.expect("struct has unnamed fields");
            let field_expr = quote_expr!(cx, $ident);

            match field.attrs.skip_serializing_if() {
                Some(path) => quote_expr!(cx, if $path($field_expr) { 0 } else { 1 }),
                None => quote_expr!(cx, 1),
            }
         })
        .fold(quote_expr!(cx, 0), |sum, expr| quote_expr!(cx, $sum + $expr));

    quote_block!(cx, {
        let mut state = try!(_serializer.serialize_struct_variant(
            $item_name,
            $variant_index,
            $variant_name,
            $len,
        ));
        $serialize_fields
        _serializer.serialize_struct_variant_end(state)
    }).unwrap()
}

fn serialize_tuple_struct_visitor(
    cx: &ExtCtxt,
    builder: &aster::AstBuilder,
    structure_ty: P<ast::Ty>,
    fields: &[Field],
    generics: &ast::Generics,
    is_enum: bool,
    func: ast::Ident,
) -> Vec<ast::Stmt> {
    fields.iter()
        .enumerate()
        .map(|(i, field)| {
            let mut field_expr = if is_enum {
                builder.expr().path().id(format!("__field{}", i)).build()
            } else {
                builder.expr().ref_().tup_field(i).self_()
            };

            let skip = field.attrs.skip_serializing_if()
                .map(|path| quote_expr!(cx, $path($field_expr)))
                .unwrap_or(quote_expr!(cx, false));

            if let Some(path) = field.attrs.serialize_with() {
                field_expr = wrap_serialize_with(cx, builder,
                    &structure_ty, generics, &field.ty, path, field_expr);
            }

            quote_stmt!(cx,
                if !$skip {
                    try!(_serializer.$func(&mut state, $field_expr));
                }
            ).unwrap()
        })
        .collect()
}

fn serialize_struct_visitor(
    cx: &ExtCtxt,
    builder: &aster::AstBuilder,
    structure_ty: P<ast::Ty>,
    fields: &[Field],
    generics: &ast::Generics,
    is_enum: bool,
    func: ast::Ident,
) -> Vec<ast::Stmt> {
    fields.iter()
        .filter(|&field| !field.attrs.skip_serializing())
        .map(|field| {
            let ident = field.ident.expect("struct has unnamed field");
            let mut field_expr = if is_enum {
                quote_expr!(cx, $ident)
            } else {
                quote_expr!(cx, &self.$ident)
            };

            let key_expr = name_expr(builder, field.attrs.name());

            let skip = field.attrs.skip_serializing_if()
                .map(|path| quote_expr!(cx, $path($field_expr)))
                .unwrap_or(quote_expr!(cx, false));

            if let Some(path) = field.attrs.serialize_with() {
                field_expr = wrap_serialize_with(cx, builder,
                    &structure_ty, generics, &field.ty, path, field_expr)
            }

            quote_stmt!(cx,
                if !$skip {
                    try!(_serializer.$func(&mut state, $key_expr, $field_expr));
                }
            ).unwrap()
        })
        .collect()
}

fn wrap_serialize_with(
    cx: &ExtCtxt,
    builder: &aster::AstBuilder,
    item_ty: &P<ast::Ty>,
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
            phantom: ::std::marker::PhantomData<$item_ty>,
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
            phantom: ::std::marker::PhantomData::<$item_ty>,
        }
    })
}

fn name_expr(
    builder: &aster::AstBuilder,
    name: &attr::Name,
) -> P<ast::Expr> {
    builder.expr().str(name.serialize_name())
}
