use syn::{self, aster, Ident};
use quote::{self, Tokens};

use bound;
use internals::ast::{Body, Field, Item, Style, Variant};
use internals::{self, attr};

use std::iter;

pub fn expand_derive_deserialize(item: &syn::MacroInput) -> Result<Tokens, String> {
    let item = {
        let ctxt = internals::Ctxt::new();
        let item = Item::from_ast(&ctxt, item);
        check_no_str(&ctxt, &item);
        try!(ctxt.check());
        item
    };

    let impl_generics = build_impl_generics(&item);

    let ty = aster::ty().path()
        .segment(item.ident.clone()).with_generics(impl_generics.clone()).build()
        .build();

    let body = deserialize_body(&item,
                                &impl_generics,
                                ty.clone());

    let where_clause = &impl_generics.where_clause;

    let dummy_const = Ident::new(format!("_IMPL_DESERIALIZE_FOR_{}", item.ident));

    Ok(quote! {
        #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
        const #dummy_const: () = {
            extern crate serde as _serde;
            #[automatically_derived]
            impl #impl_generics _serde::Deserialize for #ty #where_clause {
                fn deserialize<__D>(deserializer: __D) -> _serde::export::Result<#ty, __D::Error>
                    where __D: _serde::Deserializer
                #body
            }
        };
    })
}

// All the generics in the input, plus a bound `T: Deserialize` for each generic
// field type that will be deserialized by us, plus a bound `T: Default` for
// each generic field type that will be set to a default value.
fn build_impl_generics(item: &Item) -> syn::Generics {
    let generics = bound::without_defaults(item.generics);

    let generics = bound::with_where_predicates_from_fields(
        item, &generics,
        |attrs| attrs.de_bound());

    match item.attrs.de_bound() {
        Some(predicates) => {
            bound::with_where_predicates(&generics, predicates)
        }
        None => {
            let generics = bound::with_bound(item, &generics,
                needs_deserialize_bound,
                &aster::path().ids(&["_serde", "Deserialize"]).build());
            bound::with_bound(item, &generics,
                requires_default,
                &aster::path().global().ids(&["std", "default", "Default"]).build())
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
    item: &Item,
    impl_generics: &syn::Generics,
    ty: syn::Ty,
) -> Tokens {
    match item.body {
        Body::Enum(ref variants) => {
            deserialize_item_enum(
                &item.ident,
                impl_generics,
                ty,
                variants,
                &item.attrs)
        }
        Body::Struct(Style::Struct, ref fields) => {
            if fields.iter().any(|field| field.ident.is_none()) {
                panic!("struct has unnamed fields");
            }

            deserialize_struct(
                &item.ident,
                None,
                impl_generics,
                ty,
                fields,
                &item.attrs)
        }
        Body::Struct(Style::Tuple, ref fields) |
        Body::Struct(Style::Newtype, ref fields) => {
            if fields.iter().any(|field| field.ident.is_some()) {
                panic!("tuple struct has named fields");
            }

            deserialize_tuple(
                &item.ident,
                None,
                impl_generics,
                ty,
                fields,
                &item.attrs)
        }
        Body::Struct(Style::Unit, _) => {
            deserialize_unit_struct(
                &item.ident,
                &item.attrs)
        }
    }
}

// Build `__Visitor<A, B, ...>(PhantomData<A>, PhantomData<B>, ...)`
//
// Returns:
//
//     1. the struct declaration
//     2. the visitor type, including generics
//     3. the expression for instantiating the visitor
fn deserialize_visitor(generics: &syn::Generics) -> (Tokens, Tokens, Tokens) {
    if generics.lifetimes.is_empty() && generics.ty_params.is_empty() {
        (
            quote! {
                struct __Visitor;
            },
            quote!(__Visitor),
            quote!(__Visitor),
        )
    } else {
        let where_clause = &generics.where_clause;

        let num_phantoms = generics.lifetimes.len() + generics.ty_params.len();

        let phantom_types = generics.lifetimes.iter()
            .map(|lifetime_def| {
                let lifetime = &lifetime_def.lifetime;
                quote!(_serde::export::PhantomData<& #lifetime ()>)
            }).chain(generics.ty_params.iter()
                .map(|ty_param| {
                    let ident = &ty_param.ident;
                    quote!(_serde::export::PhantomData<#ident>)
                }));

        let all_params = generics.lifetimes.iter()
            .map(|lifetime_def| {
                let lifetime = &lifetime_def.lifetime;
                quote!(#lifetime)
            }).chain(generics.ty_params.iter()
                .map(|ty_param| {
                    let ident = &ty_param.ident;
                    quote!(#ident)
                }));

        let ty_param_idents = if generics.ty_params.is_empty() {
            None
        } else {
            let ty_param_idents = generics.ty_params.iter().map(|t| &t.ident);
            Some(quote!(::<#(#ty_param_idents),*>))
        };

        let phantom_exprs = iter::repeat(quote!(_serde::export::PhantomData)).take(num_phantoms);

        (
            quote! {
                struct __Visitor #generics ( #(#phantom_types),* ) #where_clause;
            },
            quote!(__Visitor <#(#all_params),*> ),
            quote!(__Visitor #ty_param_idents ( #(#phantom_exprs),* )),
        )
    }
}

fn deserialize_unit_struct(
    type_ident: &syn::Ident,
    item_attrs: &attr::Item,
) -> Tokens {
    let type_name = item_attrs.name().deserialize_name();

    let expecting = format!("unit struct {}", type_ident);

    quote!({
        struct __Visitor;

        impl _serde::de::Visitor for __Visitor {
            type Value = #type_ident;

            fn expecting(&self, formatter: &mut _serde::export::fmt::Formatter) -> _serde::export::fmt::Result {
                formatter.write_str(#expecting)
            }

            #[inline]
            fn visit_unit<__E>(self) -> _serde::export::Result<#type_ident, __E>
                where __E: _serde::de::Error,
            {
                Ok(#type_ident)
            }

            #[inline]
            fn visit_seq<__V>(self, _: __V) -> _serde::export::Result<#type_ident, __V::Error>
                where __V: _serde::de::SeqVisitor,
            {
                Ok(#type_ident)
            }
        }

        deserializer.deserialize_unit_struct(#type_name, __Visitor)
    })
}

fn deserialize_tuple(
    type_ident: &syn::Ident,
    variant_ident: Option<&syn::Ident>,
    impl_generics: &syn::Generics,
    ty: syn::Ty,
    fields: &[Field],
    item_attrs: &attr::Item,
) -> Tokens {
    let where_clause = &impl_generics.where_clause;

    let (visitor_item, visitor_ty, visitor_expr) = deserialize_visitor(impl_generics);

    let is_enum = variant_ident.is_some();
    let type_path = match variant_ident {
        Some(variant_ident) => quote!(#type_ident::#variant_ident),
        None => quote!(#type_ident),
    };
    let expecting = match variant_ident {
        Some(variant_ident) => format!("tuple variant {}::{}", type_ident, variant_ident),
        None => format!("tuple struct {}", type_ident),
    };

    let nfields = fields.len();

    let visit_newtype_struct = if !is_enum && nfields == 1 {
        Some(deserialize_newtype_struct(
            type_ident,
            &type_path,
            impl_generics,
            &fields[0],
        ))
    } else {
        None
    };

    let visit_seq = deserialize_seq(
        type_ident,
        &type_path,
        impl_generics,
        fields,
        false,
    );

    let dispatch = if is_enum {
        quote!(_serde::de::VariantVisitor::visit_tuple(visitor, #nfields, #visitor_expr))
    } else if nfields == 1 {
        let type_name = item_attrs.name().deserialize_name();
        quote!(deserializer.deserialize_newtype_struct(#type_name, #visitor_expr))
    } else {
        let type_name = item_attrs.name().deserialize_name();
        quote!(deserializer.deserialize_tuple_struct(#type_name, #nfields, #visitor_expr))
    };

    let all_skipped = fields.iter().all(|field| field.attrs.skip_deserializing());
    let visitor_var = if all_skipped {
        quote!(_)
    } else {
        quote!(mut visitor)
    };

    quote!({
        #visitor_item

        impl #impl_generics _serde::de::Visitor for #visitor_ty #where_clause {
            type Value = #ty;

            fn expecting(&self, formatter: &mut _serde::export::fmt::Formatter) -> _serde::export::fmt::Result {
                formatter.write_str(#expecting)
            }

            #visit_newtype_struct

            #[inline]
            fn visit_seq<__V>(self, #visitor_var: __V) -> _serde::export::Result<#ty, __V::Error>
                where __V: _serde::de::SeqVisitor
            {
                #visit_seq
            }
        }

        #dispatch
    })
}

fn deserialize_seq(
    type_ident: &syn::Ident,
    type_path: &Tokens,
    impl_generics: &syn::Generics,
    fields: &[Field],
    is_struct: bool,
) -> Tokens {
    let vars = (0..fields.len()).map(field_i as fn(_) -> _);

    let deserialized_count = fields.iter()
        .filter(|field| !field.attrs.skip_deserializing())
        .count();
    let expecting = format!("tuple of {} elements", deserialized_count);

    let mut index_in_seq = 0usize;
    let let_values = vars.clone().zip(fields)
        .map(|(var, field)| {
            if field.attrs.skip_deserializing() {
                let default = expr_is_missing(&field.attrs);
                quote! {
                    let #var = #default;
                }
            } else {
                let visit = match field.attrs.deserialize_with() {
                    None => {
                        let field_ty = &field.ty;
                        quote!(try!(visitor.visit::<#field_ty>()))
                    }
                    Some(path) => {
                        let (wrapper, wrapper_impl, wrapper_ty) = wrap_deserialize_with(
                            type_ident, impl_generics, field.ty, path);
                        quote!({
                            #wrapper
                            #wrapper_impl
                            try!(visitor.visit::<#wrapper_ty>()).map(|wrap| wrap.value)
                        })
                    }
                };
                let assign = quote! {
                    let #var = match #visit {
                        Some(value) => { value },
                        None => {
                            return Err(_serde::de::Error::invalid_length(#index_in_seq, &#expecting));
                        }
                    };
                };
                index_in_seq += 1;
                assign
            }
        });

    let result = if is_struct {
        let names = fields.iter().map(|f| &f.ident);
        quote! {
            #type_path { #( #names: #vars ),* }
        }
    } else {
        quote! {
            #type_path ( #(#vars),* )
        }
    };

    quote! {
        #(#let_values)*
        Ok(#result)
    }
}

fn deserialize_newtype_struct(
    type_ident: &syn::Ident,
    type_path: &Tokens,
    impl_generics: &syn::Generics,
    field: &Field,
) -> Tokens {
    let value = match field.attrs.deserialize_with() {
        None => {
            let field_ty = &field.ty;
            quote! {
                try!(<#field_ty as _serde::Deserialize>::deserialize(__e))
            }
        }
        Some(path) => {
            let (wrapper, wrapper_impl, wrapper_ty) = wrap_deserialize_with(
                type_ident, impl_generics, field.ty, path);
            quote!({
                #wrapper
                #wrapper_impl
                try!(<#wrapper_ty as _serde::Deserialize>::deserialize(__e)).value
            })
        }
    };
    quote! {
        #[inline]
        fn visit_newtype_struct<__E>(self, __e: __E) -> _serde::export::Result<Self::Value, __E::Error>
            where __E: _serde::Deserializer,
        {
            Ok(#type_path(#value))
        }
    }
}

fn deserialize_struct(
    type_ident: &syn::Ident,
    variant_ident: Option<&syn::Ident>,
    impl_generics: &syn::Generics,
    ty: syn::Ty,
    fields: &[Field],
    item_attrs: &attr::Item,
) -> Tokens {
    let where_clause = &impl_generics.where_clause;

    let (visitor_item, visitor_ty, visitor_expr) = deserialize_visitor(impl_generics);

    let type_path = match variant_ident {
        Some(variant_ident) => quote!(#type_ident::#variant_ident),
        None => quote!(#type_ident),
    };
    let expecting = match variant_ident {
        Some(variant_ident) => format!("struct variant {}::{}", type_ident, variant_ident),
        None => format!("struct {}", type_ident),
    };

    let visit_seq = deserialize_seq(
        type_ident,
        &type_path,
        impl_generics,
        fields,
        true,
    );

    let (field_visitor, fields_stmt, visit_map) = deserialize_struct_visitor(
        type_ident,
        type_path,
        impl_generics,
        fields,
        item_attrs,
    );

    let is_enum = variant_ident.is_some();
    let dispatch = if is_enum {
        quote! {
            _serde::de::VariantVisitor::visit_struct(visitor, FIELDS, #visitor_expr)
        }
    } else {
        let type_name = item_attrs.name().deserialize_name();
        quote! {
            deserializer.deserialize_struct(#type_name, FIELDS, #visitor_expr)
        }
    };

    let all_skipped = fields.iter().all(|field| field.attrs.skip_deserializing());
    let visitor_var = if all_skipped {
        quote!(_)
    } else {
        quote!(mut visitor)
    };

    quote!({
        #field_visitor

        #visitor_item

        impl #impl_generics _serde::de::Visitor for #visitor_ty #where_clause {
            type Value = #ty;

            fn expecting(&self, formatter: &mut _serde::export::fmt::Formatter) -> _serde::export::fmt::Result {
                formatter.write_str(#expecting)
            }

            #[inline]
            fn visit_seq<__V>(self, #visitor_var: __V) -> _serde::export::Result<#ty, __V::Error>
                where __V: _serde::de::SeqVisitor
            {
                #visit_seq
            }

            #[inline]
            fn visit_map<__V>(self, mut visitor: __V) -> _serde::export::Result<#ty, __V::Error>
                where __V: _serde::de::MapVisitor
            {
                #visit_map
            }
        }

        #fields_stmt

        #dispatch
    })
}

fn deserialize_item_enum(
    type_ident: &syn::Ident,
    impl_generics: &syn::Generics,
    ty: syn::Ty,
    variants: &[Variant],
    item_attrs: &attr::Item
) -> Tokens {
    let where_clause = &impl_generics.where_clause;

    let type_name = item_attrs.name().deserialize_name();

    let expecting = format!("enum {}", type_ident);

    let variant_names_idents: Vec<_> = variants.iter()
        .enumerate()
        .filter(|&(_, variant)| !variant.attrs.skip_deserializing())
        .map(|(i, variant)| (variant.attrs.name().deserialize_name(), field_i(i)))
        .collect();

    let variants_stmt = {
        let variant_names = variant_names_idents.iter().map(|&(ref name, _)| name);
        quote! {
            const VARIANTS: &'static [&'static str] = &[ #(#variant_names),* ];
        }
    };

    let variant_visitor = deserialize_field_visitor(
        variant_names_idents,
        item_attrs,
        true,
    );

    // Match arms to extract a variant from a string
    let variant_arms = variants.iter()
        .enumerate()
        .filter(|&(_, variant)| !variant.attrs.skip_deserializing())
        .map(|(i, variant)| {
            let variant_name = field_i(i);

            let block = deserialize_variant(
                type_ident,
                impl_generics,
                ty.clone(),
                variant,
                item_attrs,
            );

            quote! {
                (__Field::#variant_name, visitor) => #block
            }
        });

    let all_skipped = variants.iter().all(|variant| variant.attrs.skip_deserializing());
    let match_variant = if all_skipped {
        // This is an empty enum like `enum Impossible {}` or an enum in which
        // all variants have `#[serde(skip_deserializing)]`.
        quote! {
            // FIXME: Once we drop support for Rust 1.15:
            // let Err(err) = visitor.visit_variant::<__Field>();
            // Err(err)
            visitor.visit_variant::<__Field>().map(|(impossible, _)| match impossible {})
        }
    } else {
        quote! {
            match try!(visitor.visit_variant()) {
                #(#variant_arms)*
            }
        }
    };

    let (visitor_item, visitor_ty, visitor_expr) = deserialize_visitor(impl_generics);

    quote!({
        #variant_visitor

        #visitor_item

        impl #impl_generics _serde::de::Visitor for #visitor_ty #where_clause {
            type Value = #ty;

            fn expecting(&self, formatter: &mut _serde::export::fmt::Formatter) -> _serde::export::fmt::Result {
                formatter.write_str(#expecting)
            }

            fn visit_enum<__V>(self, visitor: __V) -> _serde::export::Result<#ty, __V::Error>
                where __V: _serde::de::EnumVisitor,
            {
                #match_variant
            }
        }

        #variants_stmt

        deserializer.deserialize_enum(#type_name, VARIANTS, #visitor_expr)
    })
}

fn deserialize_variant(
    type_ident: &syn::Ident,
    generics: &syn::Generics,
    ty: syn::Ty,
    variant: &Variant,
    item_attrs: &attr::Item,
) -> Tokens {
    let variant_ident = &variant.ident;

    match variant.style {
        Style::Unit => {
            quote!({
                try!(_serde::de::VariantVisitor::visit_unit(visitor));
                Ok(#type_ident::#variant_ident)
            })
        }
        Style::Newtype => {
            deserialize_newtype_variant(
                type_ident,
                variant_ident,
                generics,
                &variant.fields[0],
            )
        }
        Style::Tuple => {
            deserialize_tuple(
                type_ident,
                Some(variant_ident),
                generics,
                ty,
                &variant.fields,
                item_attrs,
            )
        }
        Style::Struct => {
            deserialize_struct(
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
    type_ident: &syn::Ident,
    variant_ident: &syn::Ident,
    impl_generics: &syn::Generics,
    field: &Field,
) -> Tokens {
    let visit = match field.attrs.deserialize_with() {
        None => {
            let field_ty = &field.ty;
            quote! {
                try!(_serde::de::VariantVisitor::visit_newtype::<#field_ty>(visitor))
            }
        }
        Some(path) => {
            let (wrapper, wrapper_impl, wrapper_ty) = wrap_deserialize_with(
                type_ident, impl_generics, field.ty, path);
            quote!({
                #wrapper
                #wrapper_impl
                try!(_serde::de::VariantVisitor::visit_newtype::<#wrapper_ty>(visitor)).value
            })
        }
    };
    quote! {
        Ok(#type_ident::#variant_ident(#visit)),
    }
}

fn deserialize_field_visitor(
    fields: Vec<(String, Ident)>,
    item_attrs: &attr::Item,
    is_variant: bool,
) -> Tokens {
    let field_strs = fields.iter().map(|&(ref name, _)| name);
    let field_bytes = fields.iter().map(|&(ref name, _)| quote::ByteStr(name));
    let field_idents: &Vec<_> = &fields.iter().map(|&(_, ref ident)| ident).collect();

    let ignore_variant = if is_variant || item_attrs.deny_unknown_fields() {
        None
    } else {
        Some(quote!(__ignore,))
    };

    let visit_index = if is_variant {
        let variant_indices = 0u32..;
        let fallthrough_msg = format!("variant index 0 <= i < {}", fields.len());
        Some(quote! {
            fn visit_u32<__E>(self, value: u32) -> _serde::export::Result<__Field, __E>
                where __E: _serde::de::Error
            {
                match value {
                    #(
                        #variant_indices => Ok(__Field::#field_idents),
                    )*
                    _ => Err(_serde::de::Error::invalid_value(
                                _serde::de::Unexpected::Unsigned(value as u64),
                                &#fallthrough_msg))
                }
            }
        })
    } else {
        None
    };

    let fallthrough_arm = if is_variant {
        quote! {
            Err(_serde::de::Error::unknown_variant(value, VARIANTS))
        }
    } else if item_attrs.deny_unknown_fields() {
        quote! {
            Err(_serde::de::Error::unknown_field(value, FIELDS))
        }
    } else {
        quote! {
            Ok(__Field::__ignore)
        }
    };

    let bytes_to_str = if is_variant || item_attrs.deny_unknown_fields() {
        Some(quote! {
            // TODO https://github.com/serde-rs/serde/issues/666
            // update this to use str::from_utf8(value).unwrap_or("���") on no_std
            let value = &_serde::export::from_utf8_lossy(value);
        })
    } else {
        None
    };

    quote! {
        #[allow(non_camel_case_types)]
        enum __Field {
            #(#field_idents,)*
            #ignore_variant
        }

        impl _serde::Deserialize for __Field {
            #[inline]
            fn deserialize<__D>(deserializer: __D) -> _serde::export::Result<__Field, __D::Error>
                where __D: _serde::Deserializer,
            {
                struct __FieldVisitor;

                impl _serde::de::Visitor for __FieldVisitor {
                    type Value = __Field;

                    fn expecting(&self, formatter: &mut _serde::export::fmt::Formatter) -> _serde::export::fmt::Result {
                        formatter.write_str("field name")
                    }

                    #visit_index

                    fn visit_str<__E>(self, value: &str) -> _serde::export::Result<__Field, __E>
                        where __E: _serde::de::Error
                    {
                        match value {
                            #(
                                #field_strs => Ok(__Field::#field_idents),
                            )*
                            _ => #fallthrough_arm
                        }
                    }

                    fn visit_bytes<__E>(self, value: &[u8]) -> _serde::export::Result<__Field, __E>
                        where __E: _serde::de::Error
                    {
                        match value {
                            #(
                                #field_bytes => Ok(__Field::#field_idents),
                            )*
                            _ => {
                                #bytes_to_str
                                #fallthrough_arm
                            }
                        }
                    }
                }

                deserializer.deserialize_struct_field(__FieldVisitor)
            }
        }
    }
}

fn deserialize_struct_visitor(
    type_ident: &syn::Ident,
    struct_path: Tokens,
    impl_generics: &syn::Generics,
    fields: &[Field],
    item_attrs: &attr::Item,
) -> (Tokens, Tokens, Tokens) {
    let field_names_idents: Vec<_> = fields.iter()
        .enumerate()
        .filter(|&(_, field)| !field.attrs.skip_deserializing())
        .map(|(i, field)| (field.attrs.name().deserialize_name(), field_i(i)))
        .collect();

    let fields_stmt = {
        let field_names = field_names_idents.iter().map(|&(ref name, _)| name);
        quote! {
            const FIELDS: &'static [&'static str] = &[ #(#field_names),* ];
        }
    };

    let field_visitor = deserialize_field_visitor(
        field_names_idents,
        item_attrs,
        false,
    );

    let visit_map = deserialize_map(
        type_ident,
        struct_path,
        impl_generics,
        fields,
        item_attrs,
    );

    (field_visitor, fields_stmt, visit_map)
}

fn deserialize_map(
    type_ident: &syn::Ident,
    struct_path: Tokens,
    impl_generics: &syn::Generics,
    fields: &[Field],
    item_attrs: &attr::Item,
) -> Tokens {
    // Create the field names for the fields.
    let fields_names: Vec<_> = fields.iter()
        .enumerate()
        .map(|(i, field)| (field, field_i(i)))
        .collect();

    // Declare each field that will be deserialized.
    let let_values = fields_names.iter()
        .filter(|&&(field, _)| !field.attrs.skip_deserializing())
        .map(|&(field, ref name)| {
            let field_ty = &field.ty;
            quote! {
                let mut #name: Option<#field_ty> = None;
            }
        });

    // Match arms to extract a value for a field.
    let value_arms = fields_names.iter()
        .filter(|&&(field, _)| !field.attrs.skip_deserializing())
        .map(|&(field, ref name)| {
            let deser_name = field.attrs.name().deserialize_name();

            let visit = match field.attrs.deserialize_with() {
                None => {
                    let field_ty = &field.ty;
                    quote! {
                        try!(visitor.visit_value::<#field_ty>())
                    }
                }
                Some(path) => {
                    let (wrapper, wrapper_impl, wrapper_ty) = wrap_deserialize_with(
                        type_ident, impl_generics, field.ty, path);
                    quote!({
                        #wrapper
                        #wrapper_impl
                        try!(visitor.visit_value::<#wrapper_ty>()).value
                    })
                }
            };
            quote! {
                __Field::#name => {
                    if #name.is_some() {
                        return Err(<__V::Error as _serde::de::Error>::duplicate_field(#deser_name));
                    }
                    #name = Some(#visit);
                }
            }
        });

    // Visit ignored values to consume them
    let ignored_arm = if item_attrs.deny_unknown_fields() {
        None
    } else {
        Some(quote! {
            _ => { let _ = try!(visitor.visit_value::<_serde::de::impls::IgnoredAny>()); }
        })
    };

    let all_skipped = fields.iter().all(|field| field.attrs.skip_deserializing());
    let match_keys = if item_attrs.deny_unknown_fields() && all_skipped {
        quote! {
            // FIXME: Once we drop support for Rust 1.15:
            // let None::<__Field> = try!(visitor.visit_key());
            try!(visitor.visit_key::<__Field>()).map(|impossible| match impossible {});
        }
    } else {
        quote! {
            while let Some(key) = try!(visitor.visit_key::<__Field>()) {
                match key {
                    #(#value_arms)*
                    #ignored_arm
                }
            }
        }
    };

    let extract_values = fields_names.iter()
        .filter(|&&(field, _)| !field.attrs.skip_deserializing())
        .map(|&(field, ref name)| {
            let missing_expr = expr_is_missing(&field.attrs);

            quote! {
                let #name = match #name {
                    Some(#name) => #name,
                    None => #missing_expr
                };
            }
        });

    let result = fields_names.iter()
        .map(|&(field, ref name)| {
            let ident = field.ident.clone().expect("struct contains unnamed fields");
            let value = if field.attrs.skip_deserializing() {
                expr_is_missing(&field.attrs)
            } else {
                quote!(#name)
            };
            quote!(#ident: #value)
        });

    quote! {
        #(#let_values)*

        #match_keys

        #(#extract_values)*

        Ok(#struct_path { #(#result),* })
    }
}

fn field_i(i: usize) -> Ident {
    Ident::new(format!("__field{}", i))
}

/// This function wraps the expression in `#[serde(deserialize_with="...")]` in
/// a trait to prevent it from accessing the internal `Deserialize` state.
fn wrap_deserialize_with(
    type_ident: &syn::Ident,
    impl_generics: &syn::Generics,
    field_ty: &syn::Ty,
    deserialize_with: &syn::Path,
) -> (Tokens, Tokens, syn::Path) {
    // Quasi-quoting doesn't do a great job of expanding generics into paths,
    // so manually build it.
    let wrapper_ty = aster::path()
        .segment("__SerdeDeserializeWithStruct")
            .with_generics(impl_generics.clone())
            .build()
        .build();

    let where_clause = &impl_generics.where_clause;

    let phantom_ty = aster::path()
        .segment(type_ident)
            .with_generics(aster::from_generics(impl_generics.clone())
                .strip_ty_params()
                .build())
            .build()
        .build();

    (
        quote! {
            struct __SerdeDeserializeWithStruct #impl_generics #where_clause {
                value: #field_ty,
                phantom: _serde::export::PhantomData<#phantom_ty>,
            }
        },
        quote! {
            impl #impl_generics _serde::Deserialize for #wrapper_ty #where_clause {
                fn deserialize<__D>(__d: __D) -> _serde::export::Result<Self, __D::Error>
                    where __D: _serde::Deserializer
                {
                    let value = try!(#deserialize_with(__d));
                    Ok(__SerdeDeserializeWithStruct {
                        value: value,
                        phantom: _serde::export::PhantomData,
                    })
                }
            }
        },
        wrapper_ty,
    )
}

fn expr_is_missing(attrs: &attr::Field) -> Tokens {
    match *attrs.default() {
        attr::FieldDefault::Default => {
            return quote!(_serde::export::Default::default());
        }
        attr::FieldDefault::Path(ref path) => {
            return quote!(#path());
        }
        attr::FieldDefault::None => { /* below */ }
    }

    let name = attrs.name().deserialize_name();
    match attrs.deserialize_with() {
        None => {
            quote! {
                try!(_serde::de::private::missing_field(#name))
            }
        }
        Some(_) => {
            quote! {
                return Err(<__V::Error as _serde::de::Error>::missing_field(#name))
            }
        }
    }
}

fn check_no_str(cx: &internals::Ctxt, item: &Item) {
    let fail = || {
        cx.error(
            "Serde does not support deserializing fields of type &str; \
             consider using String instead");
    };

    for field in item.body.all_fields() {
        if field.attrs.skip_deserializing()
            || field.attrs.deserialize_with().is_some() { continue }

        if let syn::Ty::Rptr(_, ref inner) = *field.ty {
            if let syn::Ty::Path(_, ref path) = inner.ty {
                if path.segments.len() == 1 && path.segments[0].ident == "str" {
                    fail();
                    return;
                }
            }
        }
    }
}
