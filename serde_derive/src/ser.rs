use syn::{self, aster, Ident};
use quote::Tokens;

use bound;
use internals::ast::{Body, Field, Item, Style, Variant};
use internals::{self, attr};

pub fn expand_derive_serialize(item: &syn::DeriveInput) -> Result<Tokens, String> {
    let ctxt = internals::Ctxt::new();
    let item = Item::from_ast(&ctxt, item);
    try!(ctxt.check());

    let impl_generics = build_impl_generics(&item);

    let ty = aster::ty()
        .path()
        .segment(item.ident.clone())
        .with_generics(impl_generics.clone())
        .build()
        .build();

    let body = serialize_body(&item, &impl_generics, ty.clone());

    let where_clause = &impl_generics.where_clause;

    let dummy_const = Ident::new(format!("_IMPL_SERIALIZE_FOR_{}", item.ident));

    Ok(quote! {
        #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
        const #dummy_const: () = {
            extern crate serde as _serde;
            #[automatically_derived]
            impl #impl_generics _serde::Serialize for #ty #where_clause {
                fn serialize<__S>(&self, _serializer: __S) -> _serde::export::Result<__S::Ok, __S::Error>
                    where __S: _serde::Serializer
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

    let generics =
        bound::with_where_predicates_from_fields(item, &generics, |attrs| attrs.ser_bound());

    match item.attrs.ser_bound() {
        Some(predicates) => bound::with_where_predicates(&generics, predicates),
        None => {
            bound::with_bound(item,
                              &generics,
                              needs_serialize_bound,
                              &aster::path().ids(&["_serde", "Serialize"]).build())
        }
    }
}

// Fields with a `skip_serializing` or `serialize_with` attribute are not
// serialized by us so we do not generate a bound. Fields with a `bound`
// attribute specify their own bound so we do not generate one. All other fields
// may need a `T: Serialize` bound where T is the type of the field.
fn needs_serialize_bound(attrs: &attr::Field) -> bool {
    !attrs.skip_serializing() && attrs.serialize_with().is_none() && attrs.ser_bound().is_none()
}

fn serialize_body(item: &Item, impl_generics: &syn::Generics, ty: syn::Ty) -> Tokens {
    match item.body {
        Body::Enum(ref variants) => {
            serialize_item_enum(&item.ident, impl_generics, ty, variants, &item.attrs)
        }
        Body::Struct(Style::Struct, ref fields) => {
            if fields.iter().any(|field| field.ident.is_none()) {
                panic!("struct has unnamed fields");
            }

            serialize_struct(impl_generics, ty, fields, &item.attrs)
        }
        Body::Struct(Style::Tuple, ref fields) => {
            if fields.iter().any(|field| field.ident.is_some()) {
                panic!("tuple struct has named fields");
            }

            serialize_tuple_struct(impl_generics, ty, fields, &item.attrs)
        }
        Body::Struct(Style::Newtype, ref fields) => {
            serialize_newtype_struct(impl_generics, ty, &fields[0], &item.attrs)
        }
        Body::Struct(Style::Unit, _) => serialize_unit_struct(&item.attrs),
    }
}

fn serialize_unit_struct(item_attrs: &attr::Item) -> Tokens {
    let type_name = item_attrs.name().serialize_name();

    quote! {
        _serde::Serializer::serialize_unit_struct(_serializer, #type_name)
    }
}

fn serialize_newtype_struct(impl_generics: &syn::Generics,
                            item_ty: syn::Ty,
                            field: &Field,
                            item_attrs: &attr::Item)
                            -> Tokens {
    let type_name = item_attrs.name().serialize_name();

    let mut field_expr = quote!(&self.0);
    if let Some(path) = field.attrs.serialize_with() {
        field_expr = wrap_serialize_with(&item_ty, impl_generics, field.ty, path, field_expr);
    }

    quote! {
        _serde::Serializer::serialize_newtype_struct(_serializer, #type_name, #field_expr)
    }
}

fn serialize_tuple_struct(impl_generics: &syn::Generics,
                          ty: syn::Ty,
                          fields: &[Field],
                          item_attrs: &attr::Item)
                          -> Tokens {
    let serialize_stmts =
        serialize_tuple_struct_visitor(ty.clone(),
                                       fields,
                                       impl_generics,
                                       false,
                                       quote!(_serde::ser::SerializeTupleStruct::serialize_field));

    let type_name = item_attrs.name().serialize_name();
    let len = serialize_stmts.len();
    let let_mut = mut_if(len > 0);

    quote! {
        let #let_mut __serde_state = try!(_serde::Serializer::serialize_tuple_struct(_serializer, #type_name, #len));
        #(#serialize_stmts)*
        _serde::ser::SerializeTupleStruct::end(__serde_state)
    }
}

fn serialize_struct(impl_generics: &syn::Generics,
                    ty: syn::Ty,
                    fields: &[Field],
                    item_attrs: &attr::Item)
                    -> Tokens {
    let serialize_fields =
        serialize_struct_visitor(ty.clone(),
                                 fields,
                                 impl_generics,
                                 false,
                                 quote!(_serde::ser::SerializeStruct::serialize_field));

    let type_name = item_attrs.name().serialize_name();

    let mut serialized_fields = fields.iter()
        .filter(|&field| !field.attrs.skip_serializing())
        .peekable();

    let let_mut = mut_if(serialized_fields.peek().is_some());

    let len = serialized_fields.map(|field| {
            let ident = field.ident.clone().expect("struct has unnamed fields");
            let field_expr = quote!(&self.#ident);

            match field.attrs.skip_serializing_if() {
                Some(path) => quote!(if #path(#field_expr) { 0 } else { 1 }),
                None => quote!(1),
            }
        })
        .fold(quote!(0), |sum, expr| quote!(#sum + #expr));

    quote! {
        let #let_mut __serde_state = try!(_serde::Serializer::serialize_struct(_serializer, #type_name, #len));
        #(#serialize_fields)*
        _serde::ser::SerializeStruct::end(__serde_state)
    }
}

fn serialize_item_enum(type_ident: &syn::Ident,
                       impl_generics: &syn::Generics,
                       ty: syn::Ty,
                       variants: &[Variant],
                       item_attrs: &attr::Item)
                       -> Tokens {
    let arms: Vec<_> = variants.iter()
        .enumerate()
        .map(|(variant_index, variant)| {
            serialize_variant(type_ident,
                              impl_generics,
                              ty.clone(),
                              variant,
                              variant_index,
                              item_attrs)
        })
        .collect();

    quote! {
        match *self {
            #(#arms)*
        }
    }
}

fn serialize_variant(type_ident: &syn::Ident,
                     generics: &syn::Generics,
                     ty: syn::Ty,
                     variant: &Variant,
                     variant_index: usize,
                     item_attrs: &attr::Item)
                     -> Tokens {
    let variant_ident = variant.ident.clone();

    if variant.attrs.skip_serializing() {
        let skipped_msg = format!("the enum variant {}::{} cannot be serialized",
                                  type_ident, variant_ident);
        let skipped_err = quote! {
            _serde::export::Err(_serde::ser::Error::custom(#skipped_msg))
        };
        let fields_pat = match variant.style {
            Style::Unit => quote!(),
            Style::Newtype | Style::Tuple => quote!( (..) ),
            Style::Struct => quote!( {..} ),
        };
        quote! {
            #type_ident::#variant_ident #fields_pat => #skipped_err,
        }
    } else {
        // variant wasn't skipped
        let case = match variant.style {
            Style::Unit => {
                quote! {
                    #type_ident::#variant_ident
                }
            }
            Style::Newtype => {
                quote! {
                    #type_ident::#variant_ident(ref __simple_value)
                }
            }
            Style::Tuple => {
                let field_names = (0..variant.fields.len())
                    .map(|i| Ident::new(format!("__field{}", i)));
                quote! {
                    #type_ident::#variant_ident(#(ref #field_names),*)
                }
            }
            Style::Struct => {
                let fields = variant.fields
                    .iter()
                    .map(|f| f.ident.clone().expect("struct variant has unnamed fields"));
                quote! {
                    #type_ident::#variant_ident { #(ref #fields),* }
                }
            }
        };

        let body = match *item_attrs.tag() {
            attr::EnumTag::External => {
                serialize_externally_tagged_variant(generics,
                                                    ty,
                                                    variant,
                                                    variant_index,
                                                    item_attrs)
            }
            attr::EnumTag::Internal(ref tag) => {
                serialize_internally_tagged_variant(type_ident.as_ref(),
                                                    variant_ident.as_ref(),
                                                    generics,
                                                    ty,
                                                    variant,
                                                    item_attrs,
                                                    tag)
            }
            attr::EnumTag::None => serialize_untagged_variant(generics, ty, variant, item_attrs),
        };

        quote! {
            #case => #body
        }
    }
}

fn serialize_externally_tagged_variant(generics: &syn::Generics,
                                       ty: syn::Ty,
                                       variant: &Variant,
                                       variant_index: usize,
                                       item_attrs: &attr::Item)
                                       -> Tokens {
    let type_name = item_attrs.name().serialize_name();
    let variant_name = variant.attrs.name().serialize_name();

    match variant.style {
        Style::Unit => {
            quote! {
                _serde::Serializer::serialize_unit_variant(
                    _serializer,
                    #type_name,
                    #variant_index,
                    #variant_name,
                ),
            }
        }
        Style::Newtype => {
            let field = &variant.fields[0];
            let mut field_expr = quote!(__simple_value);
            if let Some(path) = field.attrs.serialize_with() {
                field_expr = wrap_serialize_with(&ty, generics, field.ty, path, field_expr);
            }

            quote! {
                _serde::Serializer::serialize_newtype_variant(
                    _serializer,
                    #type_name,
                    #variant_index,
                    #variant_name,
                    #field_expr,
                ),
            }
        }
        Style::Tuple => {
            let block = serialize_tuple_variant(TupleVariant::ExternallyTagged {
                                                    type_name: type_name,
                                                    variant_index: variant_index,
                                                    variant_name: variant_name,
                                                },
                                                generics,
                                                ty,
                                                &variant.fields);

            quote! {
                { #block }
            }
        }
        Style::Struct => {
            let block = serialize_struct_variant(StructVariant::ExternallyTagged {
                                                     variant_index: variant_index,
                                                     variant_name: variant_name,
                                                 },
                                                 generics,
                                                 ty,
                                                 &variant.fields,
                                                 item_attrs);

            quote! {
                { #block }
            }
        }
    }
}

fn serialize_internally_tagged_variant(type_ident: &str,
                                       variant_ident: &str,
                                       generics: &syn::Generics,
                                       ty: syn::Ty,
                                       variant: &Variant,
                                       item_attrs: &attr::Item,
                                       tag: &str)
                                       -> Tokens {
    let type_name = item_attrs.name().serialize_name();
    let variant_name = variant.attrs.name().serialize_name();

    match variant.style {
        Style::Unit => {
            quote!({
                let mut __struct = try!(_serde::Serializer::serialize_struct(
                    _serializer, #type_name, 1));
                try!(_serde::ser::SerializeStruct::serialize_field(
                    &mut __struct, #tag, #variant_name));
                _serde::ser::SerializeStruct::end(__struct)
            })
        }
        Style::Newtype => {
            let field = &variant.fields[0];
            let mut field_expr = quote!(__simple_value);
            if let Some(path) = field.attrs.serialize_with() {
                field_expr = wrap_serialize_with(&ty, generics, field.ty, path, field_expr);
            }

            quote! {
                _serde::ser::private::serialize_tagged_newtype(
                    _serializer,
                    #type_ident,
                    #variant_ident,
                    #tag,
                    #variant_name,
                    #field_expr,
                ),
            }
        }
        Style::Struct => {
            let block = serialize_struct_variant(StructVariant::InternallyTagged {
                                                     tag: tag,
                                                     variant_name: variant_name,
                                                 },
                                                 generics,
                                                 ty,
                                                 &variant.fields,
                                                 item_attrs);

            quote! {
                { #block }
            }
        }
        Style::Tuple => unreachable!("checked in serde_codegen_internals"),
    }
}

fn serialize_untagged_variant(generics: &syn::Generics,
                              ty: syn::Ty,
                              variant: &Variant,
                              item_attrs: &attr::Item)
                              -> Tokens {
    match variant.style {
        Style::Unit => {
            quote! {
                _serde::Serializer::serialize_unit(_serializer),
            }
        }
        Style::Newtype => {
            let field = &variant.fields[0];
            let mut field_expr = quote!(__simple_value);
            if let Some(path) = field.attrs.serialize_with() {
                field_expr = wrap_serialize_with(&ty, generics, field.ty, path, field_expr);
            }

            quote! {
                _serde::Serialize::serialize(#field_expr, _serializer),
            }
        }
        Style::Tuple => {
            let block =
                serialize_tuple_variant(TupleVariant::Untagged, generics, ty, &variant.fields);

            quote! {
                { #block }
            }
        }
        Style::Struct => {
            let block = serialize_struct_variant(StructVariant::Untagged,
                                                 generics,
                                                 ty,
                                                 &variant.fields,
                                                 item_attrs);

            quote! {
                { #block }
            }
        }
    }
}

enum TupleVariant {
    ExternallyTagged {
        type_name: String,
        variant_index: usize,
        variant_name: String,
    },
    Untagged,
}

fn serialize_tuple_variant(context: TupleVariant,
                           generics: &syn::Generics,
                           structure_ty: syn::Ty,
                           fields: &[Field])
                           -> Tokens {
    let method = match context {
        TupleVariant::ExternallyTagged { .. } => {
            quote!(_serde::ser::SerializeTupleVariant::serialize_field)
        }
        TupleVariant::Untagged => quote!(_serde::ser::SerializeTuple::serialize_element),
    };

    let serialize_stmts =
        serialize_tuple_struct_visitor(structure_ty, fields, generics, true, method);

    let len = serialize_stmts.len();
    let let_mut = mut_if(len > 0);

    match context {
        TupleVariant::ExternallyTagged { type_name, variant_index, variant_name } => {
            quote! {
                let #let_mut __serde_state = try!(_serde::Serializer::serialize_tuple_variant(
                    _serializer,
                    #type_name,
                    #variant_index,
                    #variant_name,
                    #len));
                #(#serialize_stmts)*
                _serde::ser::SerializeTupleVariant::end(__serde_state)
            }
        }
        TupleVariant::Untagged => {
            quote! {
                let #let_mut __serde_state = try!(_serde::Serializer::serialize_tuple(
                    _serializer,
                    #len));
                #(#serialize_stmts)*
                _serde::ser::SerializeTuple::end(__serde_state)
            }
        }
    }
}

enum StructVariant<'a> {
    ExternallyTagged {
        variant_index: usize,
        variant_name: String,
    },
    InternallyTagged { tag: &'a str, variant_name: String },
    Untagged,
}

fn serialize_struct_variant<'a>(context: StructVariant<'a>,
                                generics: &syn::Generics,
                                ty: syn::Ty,
                                fields: &[Field],
                                item_attrs: &attr::Item)
                                -> Tokens {
    let method = match context {
        StructVariant::ExternallyTagged { .. } => {
            quote!(_serde::ser::SerializeStructVariant::serialize_field)
        }
        StructVariant::InternallyTagged { .. } |
        StructVariant::Untagged => quote!(_serde::ser::SerializeStruct::serialize_field),
    };

    let serialize_fields = serialize_struct_visitor(ty.clone(), fields, generics, true, method);

    let item_name = item_attrs.name().serialize_name();

    let mut serialized_fields = fields.iter()
        .filter(|&field| !field.attrs.skip_serializing())
        .peekable();

    let let_mut = mut_if(serialized_fields.peek().is_some());

    let len = serialized_fields.map(|field| {
            let ident = field.ident.clone().expect("struct has unnamed fields");

            match field.attrs.skip_serializing_if() {
                Some(path) => quote!(if #path(#ident) { 0 } else { 1 }),
                None => quote!(1),
            }
        })
        .fold(quote!(0), |sum, expr| quote!(#sum + #expr));

    match context {
        StructVariant::ExternallyTagged { variant_index, variant_name } => {
            quote! {
                let #let_mut __serde_state = try!(_serde::Serializer::serialize_struct_variant(
                    _serializer,
                    #item_name,
                    #variant_index,
                    #variant_name,
                    #len,
                ));
                #(#serialize_fields)*
                _serde::ser::SerializeStructVariant::end(__serde_state)
            }
        }
        StructVariant::InternallyTagged { tag, variant_name } => {
            quote! {
                let mut __serde_state = try!(_serde::Serializer::serialize_struct(
                    _serializer,
                    #item_name,
                    #len + 1,
                ));
                try!(_serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    #tag,
                    #variant_name,
                ));
                #(#serialize_fields)*
                _serde::ser::SerializeStruct::end(__serde_state)
            }
        }
        StructVariant::Untagged => {
            quote! {
                let #let_mut __serde_state = try!(_serde::Serializer::serialize_struct(
                    _serializer,
                    #item_name,
                    #len,
                ));
                #(#serialize_fields)*
                _serde::ser::SerializeStruct::end(__serde_state)
            }
        }
    }
}

fn serialize_tuple_struct_visitor(structure_ty: syn::Ty,
                                  fields: &[Field],
                                  generics: &syn::Generics,
                                  is_enum: bool,
                                  func: Tokens)
                                  -> Vec<Tokens> {
    fields.iter()
        .enumerate()
        .map(|(i, field)| {
            let mut field_expr = if is_enum {
                let id = Ident::new(format!("__field{}", i));
                quote!(#id)
            } else {
                let i = Ident::new(i);
                quote!(&self.#i)
            };

            let skip = field.attrs
                .skip_serializing_if()
                .map(|path| quote!(#path(#field_expr)));

            if let Some(path) = field.attrs.serialize_with() {
                field_expr =
                    wrap_serialize_with(&structure_ty, generics, field.ty, path, field_expr);
            }

            let ser = quote! {
                try!(#func(&mut __serde_state, #field_expr));
            };

            match skip {
                None => ser,
                Some(skip) => quote!(if !#skip { #ser }),
            }
        })
        .collect()
}

fn serialize_struct_visitor(structure_ty: syn::Ty,
                            fields: &[Field],
                            generics: &syn::Generics,
                            is_enum: bool,
                            func: Tokens)
                            -> Vec<Tokens> {
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

            let skip = field.attrs
                .skip_serializing_if()
                .map(|path| quote!(#path(#field_expr)));

            if let Some(path) = field.attrs.serialize_with() {
                field_expr =
                    wrap_serialize_with(&structure_ty, generics, field.ty, path, field_expr)
            }

            let ser = quote! {
                try!(#func(&mut __serde_state, #key_expr, #field_expr));
            };

            match skip {
                None => ser,
                Some(skip) => quote!(if !#skip { #ser }),
            }
        })
        .collect()
}

fn wrap_serialize_with(item_ty: &syn::Ty,
                       generics: &syn::Generics,
                       field_ty: &syn::Ty,
                       path: &syn::Path,
                       value: Tokens)
                       -> Tokens {
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
            phantom: _serde::export::PhantomData<#item_ty>,
        }

        impl #wrapper_generics _serde::Serialize for #wrapper_ty #where_clause {
            fn serialize<__S>(&self, __s: __S) -> _serde::export::Result<__S::Ok, __S::Error>
                where __S: _serde::Serializer
            {
                #path(self.value, __s)
            }
        }

        &__SerializeWith {
            value: #value,
            phantom: _serde::export::PhantomData::<#item_ty>,
        }
    })
}

// Serialization of an empty struct results in code like:
//
//     let mut __serde_state = try!(serializer.serialize_struct("S", 0));
//     _serde::ser::SerializeStruct::end(__serde_state)
//
// where we want to omit the `mut` to avoid a warning.
fn mut_if(is_mut: bool) -> Option<Tokens> {
    if is_mut { Some(quote!(mut)) } else { None }
}
