use syn::{self, aster};
use quote::Tokens;

use bound;
use internals::ast::{Body, Field, Item, Style, Variant};
use internals::{self, attr};

pub fn expand_derive_serialize(item: &syn::MacroInput) -> Result<Tokens, String> {
    let ctxt = internals::Ctxt::new();
    let item = Item::from_ast(&ctxt, item);
    try!(ctxt.check());

    let impl_generics = build_impl_generics(&item);

    let ty = aster::ty().path()
        .segment(item.ident.clone()).with_generics(impl_generics.clone()).build()
        .build();

    let body = serialize_body(&item,
                              &impl_generics,
                              ty.clone());

    let where_clause = &impl_generics.where_clause;

    let dummy_const = aster::id(format!("_IMPL_SERIALIZE_FOR_{}", item.ident));

    Ok(quote! {
        #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
        const #dummy_const: () = {
            extern crate serde as _serde;
            #[automatically_derived]
            impl #impl_generics _serde::ser::Serialize for #ty #where_clause {
                fn serialize<__S>(&self, _serializer: &mut __S) -> ::std::result::Result<(), __S::Error>
                    where __S: _serde::ser::Serializer
                {
                    #body
                }
            }
        };
    })
}

// All the generics in the input, plus a bound `T: Serialize` for each generic
// field type that will be serialized by us.
fn build_impl_generics(item: &Item) -> syn::Generics {
    let generics = bound::without_defaults(item.generics);

    let generics = bound::with_where_predicates_from_fields(
        item, &generics,
        |attrs| attrs.ser_bound());

    match item.attrs.ser_bound() {
        Some(predicates) => {
            bound::with_where_predicates(&generics, predicates)
        }
        None => {
            bound::with_bound(item, &generics,
                needs_serialize_bound,
                &aster::path().ids(&["_serde", "ser", "Serialize"]).build())
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
    item: &Item,
    impl_generics: &syn::Generics,
    ty: syn::Ty,
) -> Tokens {
    match item.body {
        Body::Enum(ref variants) => {
            serialize_item_enum(
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

            serialize_struct(
                impl_generics,
                ty,
                fields,
                &item.attrs)
        }
        Body::Struct(Style::Tuple, ref fields) => {
            if fields.iter().any(|field| field.ident.is_some()) {
                panic!("tuple struct has named fields");
            }

            serialize_tuple_struct(
                impl_generics,
                ty,
                fields,
                &item.attrs)
        }
        Body::Struct(Style::Newtype, ref fields) => {
            serialize_newtype_struct(
                impl_generics,
                ty,
                &fields[0],
                &item.attrs)
        }
        Body::Struct(Style::Unit, _) => {
            serialize_unit_struct(
                &item.attrs)
        }
    }
}

fn serialize_unit_struct(item_attrs: &attr::Item) -> Tokens {
    let type_name = item_attrs.name().serialize_name();

    quote! {
        _serializer.serialize_unit_struct(#type_name)
    }
}

fn serialize_newtype_struct(
    impl_generics: &syn::Generics,
    item_ty: syn::Ty,
    field: &Field,
    item_attrs: &attr::Item,
) -> Tokens {
    let type_name = item_attrs.name().serialize_name();

    let mut field_expr = quote!(&self.0);
    if let Some(path) = field.attrs.serialize_with() {
        field_expr = wrap_serialize_with(
            &item_ty, impl_generics, field.ty, path, field_expr);
    }

    quote! {
        _serializer.serialize_newtype_struct(#type_name, #field_expr)
    }
}

fn serialize_tuple_struct(
    impl_generics: &syn::Generics,
    ty: syn::Ty,
    fields: &[Field],
    item_attrs: &attr::Item,
) -> Tokens {
    let serialize_stmts = serialize_tuple_struct_visitor(
        ty.clone(),
        fields,
        impl_generics,
        false,
        aster::id("serialize_tuple_struct_elt"),
    );

    let type_name = item_attrs.name().serialize_name();
    let len = serialize_stmts.len();
    let let_mut = mut_if(len > 0);

    quote! {
        let #let_mut __serde_state = try!(_serializer.serialize_tuple_struct(#type_name, #len));
        #(#serialize_stmts)*
        _serializer.serialize_tuple_struct_end(__serde_state)
    }
}

fn serialize_struct(
    impl_generics: &syn::Generics,
    ty: syn::Ty,
    fields: &[Field],
    item_attrs: &attr::Item,
) -> Tokens {
    let serialize_fields = serialize_struct_visitor(
        ty.clone(),
        fields,
        impl_generics,
        false,
        aster::id("serialize_struct_elt"),
    );

    let type_name = item_attrs.name().serialize_name();

    let mut serialized_fields = fields.iter()
        .filter(|&field| !field.attrs.skip_serializing())
        .peekable();

    let let_mut = mut_if(serialized_fields.peek().is_some());

    let len = serialized_fields
        .map(|field| {
            let ident = field.ident.clone().expect("struct has unnamed fields");
            let field_expr = quote!(&self.#ident);

            match field.attrs.skip_serializing_if() {
                Some(path) => quote!(if #path(#field_expr) { 0 } else { 1 }),
                None => quote!(1),
            }
         })
        .fold(quote!(0), |sum, expr| quote!(#sum + #expr));

    quote! {
        let #let_mut __serde_state = try!(_serializer.serialize_struct(#type_name, #len));
        #(#serialize_fields)*
        _serializer.serialize_struct_end(__serde_state)
    }
}

fn serialize_item_enum(
    type_ident: &syn::Ident,
    impl_generics: &syn::Generics,
    ty: syn::Ty,
    variants: &[Variant],
    item_attrs: &attr::Item,
) -> Tokens {
    let arms: Vec<_> =
        variants.iter()
            .enumerate()
            .map(|(variant_index, variant)| {
                serialize_variant(
                    type_ident,
                    impl_generics,
                    ty.clone(),
                    variant,
                    variant_index,
                    item_attrs,
                )
            })
            .collect();

    quote! {
        match *self {
            #(#arms)*
        }
    }
}

fn serialize_variant(
    type_ident: &syn::Ident,
    generics: &syn::Generics,
    ty: syn::Ty,
    variant: &Variant,
    variant_index: usize,
    item_attrs: &attr::Item,
) -> Tokens {
    let type_name = item_attrs.name().serialize_name();

    let variant_ident = variant.ident.clone();
    let variant_name = variant.attrs.name().serialize_name();

    match variant.style {
        Style::Unit => {
            quote! {
                #type_ident::#variant_ident =>
                    _serde::ser::Serializer::serialize_unit_variant(
                        _serializer,
                        #type_name,
                        #variant_index,
                        #variant_name,
                    ),
            }
        },
        Style::Newtype => {
            let block = serialize_newtype_variant(
                type_name,
                variant_index,
                variant_name,
                ty,
                generics,
                &variant.fields[0],
            );

            quote! {
                #type_ident::#variant_ident(ref __simple_value) => #block,
            }
        },
        Style::Tuple => {
            let field_names: Vec<Tokens> = (0 .. variant.fields.len())
                .map(|i| {
                    let id = aster::id(format!("__field{}", i));
                    quote!(ref #id)
                })
                .collect();

            let pat = quote!(#type_ident::#variant_ident(#(#field_names),*));

            let block = serialize_tuple_variant(
                type_name,
                variant_index,
                variant_name,
                generics,
                ty,
                &variant.fields,
            );

            quote! {
                #pat => { #block }
            }
        }
        Style::Struct => {
            let fields = variant.fields.iter().map(|field| {
                let id = match field.ident {
                    Some(ref name) => name.clone(),
                    None => panic!("struct variant has unnamed fields"),
                };
                quote!(ref #id)
            });
            let pat = quote!(#type_ident::#variant_ident { #(#fields),* });

            let block = serialize_struct_variant(
                variant_index,
                variant_name,
                generics,
                ty,
                &variant.fields,
                item_attrs,
            );

            quote! {
                #pat => { #block }
            }
        }
    }
}

fn serialize_newtype_variant(
    type_name: String,
    variant_index: usize,
    variant_name: String,
    item_ty: syn::Ty,
    generics: &syn::Generics,
    field: &Field,
) -> Tokens {
    let mut field_expr = quote!(__simple_value);
    if let Some(path) = field.attrs.serialize_with() {
        field_expr = wrap_serialize_with(
            &item_ty, generics, field.ty, path, field_expr);
    }

    quote! {
        _serde::ser::Serializer::serialize_newtype_variant(
            _serializer,
            #type_name,
            #variant_index,
            #variant_name,
            #field_expr,
        )
    }
}

fn serialize_tuple_variant(
    type_name: String,
    variant_index: usize,
    variant_name: String,
    generics: &syn::Generics,
    structure_ty: syn::Ty,
    fields: &[Field],
) -> Tokens {
    let serialize_stmts = serialize_tuple_struct_visitor(
        structure_ty,
        fields,
        generics,
        true,
        aster::id("serialize_tuple_variant_elt"),
    );

    let len = serialize_stmts.len();
    let let_mut = mut_if(len > 0);

    quote! {
        let #let_mut __serde_state = try!(_serializer.serialize_tuple_variant(
            #type_name,
            #variant_index,
            #variant_name,
            #len));
        #(#serialize_stmts)*
        _serializer.serialize_tuple_variant_end(__serde_state)
    }
}

fn serialize_struct_variant(
    variant_index: usize,
    variant_name: String,
    generics: &syn::Generics,
    ty: syn::Ty,
    fields: &[Field],
    item_attrs: &attr::Item,
) -> Tokens {
    let serialize_fields = serialize_struct_visitor(
        ty.clone(),
        fields,
        generics,
        true,
        aster::id("serialize_struct_variant_elt"),
    );

    let item_name = item_attrs.name().serialize_name();

    let mut serialized_fields = fields.iter()
        .filter(|&field| !field.attrs.skip_serializing())
        .peekable();

    let let_mut = mut_if(serialized_fields.peek().is_some());

    let len = serialized_fields
        .map(|field| {
            let ident = field.ident.clone().expect("struct has unnamed fields");

            match field.attrs.skip_serializing_if() {
                Some(path) => quote!(if #path(#ident) { 0 } else { 1 }),
                None => quote!(1),
            }
         })
        .fold(quote!(0), |sum, expr| quote!(#sum + #expr));

    quote! {
        let #let_mut __serde_state = try!(_serializer.serialize_struct_variant(
            #item_name,
            #variant_index,
            #variant_name,
            #len,
        ));
        #(#serialize_fields)*
        _serializer.serialize_struct_variant_end(__serde_state)
    }
}

fn serialize_tuple_struct_visitor(
    structure_ty: syn::Ty,
    fields: &[Field],
    generics: &syn::Generics,
    is_enum: bool,
    func: syn::Ident,
) -> Vec<Tokens> {
    fields.iter()
        .enumerate()
        .map(|(i, field)| {
            let mut field_expr = if is_enum {
                let id = aster::id(format!("__field{}", i));
                quote!(#id)
            } else {
                let i = aster::id(i);
                quote!(&self.#i)
            };

            let skip = field.attrs.skip_serializing_if()
                .map(|path| quote!(#path(#field_expr)));

            if let Some(path) = field.attrs.serialize_with() {
                field_expr = wrap_serialize_with(
                    &structure_ty, generics, field.ty, path, field_expr);
            }

            let ser = quote! {
                try!(_serializer.#func(&mut __serde_state, #field_expr));
            };

            match skip {
                None => ser,
                Some(skip) => quote!(if !#skip { #ser }),
            }
        })
        .collect()
}

fn serialize_struct_visitor(
    structure_ty: syn::Ty,
    fields: &[Field],
    generics: &syn::Generics,
    is_enum: bool,
    func: syn::Ident,
) -> Vec<Tokens> {
    fields.iter()
        .filter(|&field| !field.attrs.skip_serializing())
        .map(|field| {
            let ident = field.ident.clone().expect("struct has unnamed field");
            let mut field_expr = if is_enum {
                quote!(#ident)
            } else {
                quote!(&self.#ident)
            };

            let key_expr = field.attrs.name().serialize_name();

            let skip = field.attrs.skip_serializing_if()
                .map(|path| quote!(#path(#field_expr)));

            if let Some(path) = field.attrs.serialize_with() {
                field_expr = wrap_serialize_with(
                    &structure_ty, generics, field.ty, path, field_expr)
            }

            let ser = quote! {
                try!(_serializer.#func(&mut __serde_state, #key_expr, #field_expr));
            };

            match skip {
                None => ser,
                Some(skip) => quote!(if !#skip { #ser }),
            }
        })
        .collect()
}

fn wrap_serialize_with(
    item_ty: &syn::Ty,
    generics: &syn::Generics,
    field_ty: &syn::Ty,
    path: &syn::Path,
    value: Tokens,
) -> Tokens {
    let where_clause = &generics.where_clause;

    let wrapper_generics = aster::from_generics(generics.clone())
        .add_lifetime_bound("'__a")
        .lifetime_name("'__a")
        .build();

    let wrapper_ty = aster::path()
        .segment("__SerializeWith")
            .with_generics(wrapper_generics.clone())
            .build()
        .build();

    quote!({
        struct __SerializeWith #wrapper_generics #where_clause {
            value: &'__a #field_ty,
            phantom: ::std::marker::PhantomData<#item_ty>,
        }

        impl #wrapper_generics _serde::ser::Serialize for #wrapper_ty #where_clause {
            fn serialize<__S>(&self, __s: &mut __S) -> ::std::result::Result<(), __S::Error>
                where __S: _serde::ser::Serializer
            {
                #path(self.value, __s)
            }
        }

        __SerializeWith {
            value: #value,
            phantom: ::std::marker::PhantomData::<#item_ty>,
        }
    })
}

// Serialization of an empty struct results in code like:
//
//     let mut __serde_state = try!(serializer.serialize_struct("S", 0));
//     serializer.serialize_struct_end(__serde_state)
//
// where we want to omit the `mut` to avoid a warning.
fn mut_if(is_mut: bool) -> Option<Tokens> {
    if is_mut {
        Some(quote!(mut))
    } else {
        None
    }
}
