use syn::{self, Ident};
use quote::Tokens;

use bound;
use fragment::{Fragment, Stmts, Match};
use internals::ast::{Body, Field, Item, Style, Variant};
use internals::{self, attr};

pub fn expand_derive_serialize(item: &syn::DeriveInput) -> Result<Tokens, String> {
    let ctxt = internals::Ctxt::new();
    let item = Item::from_ast(&ctxt, item);
    try!(ctxt.check());

    let ident = &item.ident;
    let generics = build_generics(&item);
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    let dummy_const = Ident::new(format!("_IMPL_SERIALIZE_FOR_{}", ident));
    let body = Stmts(serialize_body(&item, &generics));

    Ok(quote! {
        #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
        const #dummy_const: () = {
            extern crate serde as _serde;
            #[automatically_derived]
            impl #impl_generics _serde::Serialize for #ident #ty_generics #where_clause {
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
fn build_generics(item: &Item) -> syn::Generics {
    let generics = bound::without_defaults(item.generics);

    let generics =
        bound::with_where_predicates_from_fields(item, &generics, attr::Field::ser_bound);

    match item.attrs.ser_bound() {
        Some(predicates) => bound::with_where_predicates(&generics, predicates),
        None => {
            bound::with_bound(item,
                              &generics,
                              needs_serialize_bound,
                              &path!(_serde::Serialize))
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

fn serialize_body(item: &Item, generics: &syn::Generics) -> Fragment {
    match item.body {
        Body::Enum(ref variants) => {
            serialize_item_enum(&item.ident, generics, variants, &item.attrs)
        }
        Body::Struct(Style::Struct, ref fields) => {
            if fields.iter().any(|field| field.ident.is_none()) {
                panic!("struct has unnamed fields");
            }

            serialize_struct(&item.ident, generics, fields, &item.attrs)
        }
        Body::Struct(Style::Tuple, ref fields) => {
            if fields.iter().any(|field| field.ident.is_some()) {
                panic!("tuple struct has named fields");
            }

            serialize_tuple_struct(&item.ident, generics, fields, &item.attrs)
        }
        Body::Struct(Style::Newtype, ref fields) => {
            serialize_newtype_struct(&item.ident, generics, &fields[0], &item.attrs)
        }
        Body::Struct(Style::Unit, _) => serialize_unit_struct(&item.attrs),
    }
}

fn serialize_unit_struct(item_attrs: &attr::Item) -> Fragment {
    let type_name = item_attrs.name().serialize_name();

    quote_expr! {
        _serde::Serializer::serialize_unit_struct(_serializer, #type_name)
    }
}

fn serialize_newtype_struct(ident: &syn::Ident,
                            generics: &syn::Generics,
                            field: &Field,
                            item_attrs: &attr::Item)
                            -> Fragment {
    let type_name = item_attrs.name().serialize_name();

    let mut field_expr = quote!(&self.0);
    if let Some(path) = field.attrs.serialize_with() {
        field_expr = wrap_serialize_with(ident, generics, field.ty, path, field_expr);
    }

    quote_expr! {
        _serde::Serializer::serialize_newtype_struct(_serializer, #type_name, #field_expr)
    }
}

fn serialize_tuple_struct(ident: &syn::Ident,
                          generics: &syn::Generics,
                          fields: &[Field],
                          item_attrs: &attr::Item)
                          -> Fragment {
    let serialize_stmts =
        serialize_tuple_struct_visitor(ident,
                                       fields,
                                       generics,
                                       false,
                                       quote!(_serde::ser::SerializeTupleStruct::serialize_field));

    let type_name = item_attrs.name().serialize_name();
    let len = serialize_stmts.len();
    let let_mut = mut_if(len > 0);

    quote_block! {
        let #let_mut __serde_state = try!(_serde::Serializer::serialize_tuple_struct(_serializer, #type_name, #len));
        #(#serialize_stmts)*
        _serde::ser::SerializeTupleStruct::end(__serde_state)
    }
}

fn serialize_struct(ident: &syn::Ident,
                    generics: &syn::Generics,
                    fields: &[Field],
                    item_attrs: &attr::Item)
                    -> Fragment {
    let serialize_fields =
        serialize_struct_visitor(ident,
                                 fields,
                                 generics,
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

    quote_block! {
        let #let_mut __serde_state = try!(_serde::Serializer::serialize_struct(_serializer, #type_name, #len));
        #(#serialize_fields)*
        _serde::ser::SerializeStruct::end(__serde_state)
    }
}

fn serialize_item_enum(ident: &syn::Ident,
                       generics: &syn::Generics,
                       variants: &[Variant],
                       item_attrs: &attr::Item)
                       -> Fragment {
    let arms: Vec<_> = variants.iter()
        .enumerate()
        .map(|(variant_index, variant)| {
            serialize_variant(ident,
                              generics,
                              variant,
                              variant_index,
                              item_attrs)
        })
        .collect();

    quote_expr! {
        match *self {
            #(#arms)*
        }
    }
}

fn serialize_variant(ident: &syn::Ident,
                     generics: &syn::Generics,
                     variant: &Variant,
                     variant_index: usize,
                     item_attrs: &attr::Item)
                     -> Tokens {
    let variant_ident = variant.ident.clone();

    if variant.attrs.skip_serializing() {
        let skipped_msg = format!("the enum variant {}::{} cannot be serialized",
                                  ident, variant_ident);
        let skipped_err = quote! {
            _serde::export::Err(_serde::ser::Error::custom(#skipped_msg))
        };
        let fields_pat = match variant.style {
            Style::Unit => quote!(),
            Style::Newtype | Style::Tuple => quote!( (..) ),
            Style::Struct => quote!( {..} ),
        };
        quote! {
            #ident::#variant_ident #fields_pat => #skipped_err,
        }
    } else {
        // variant wasn't skipped
        let case = match variant.style {
            Style::Unit => {
                quote! {
                    #ident::#variant_ident
                }
            }
            Style::Newtype => {
                quote! {
                    #ident::#variant_ident(ref __field0)
                }
            }
            Style::Tuple => {
                let field_names = (0..variant.fields.len())
                    .map(|i| Ident::new(format!("__field{}", i)));
                quote! {
                    #ident::#variant_ident(#(ref #field_names),*)
                }
            }
            Style::Struct => {
                let fields = variant.fields
                    .iter()
                    .map(|f| f.ident.clone().expect("struct variant has unnamed fields"));
                quote! {
                    #ident::#variant_ident { #(ref #fields),* }
                }
            }
        };

        let body = Match(match *item_attrs.tag() {
            attr::EnumTag::External => {
                serialize_externally_tagged_variant(ident,
                                                    generics,
                                                    variant,
                                                    variant_index,
                                                    item_attrs)
            }
            attr::EnumTag::Internal { ref tag } => {
                serialize_internally_tagged_variant(ident,
                                                    generics,
                                                    variant,
                                                    item_attrs,
                                                    tag)
            }
            attr::EnumTag::Adjacent { ref tag, ref content } => {
                serialize_adjacently_tagged_variant(ident,
                                                    generics,
                                                    variant,
                                                    item_attrs,
                                                    tag,
                                                    content)
            }
            attr::EnumTag::None => serialize_untagged_variant(ident, generics, variant, item_attrs),
        });

        quote! {
            #case => #body
        }
    }
}

fn serialize_externally_tagged_variant(ident: &syn::Ident,
                                       generics: &syn::Generics,
                                       variant: &Variant,
                                       variant_index: usize,
                                       item_attrs: &attr::Item)
                                       -> Fragment {
    let type_name = item_attrs.name().serialize_name();
    let variant_name = variant.attrs.name().serialize_name();

    match variant.style {
        Style::Unit => {
            quote_expr! {
                _serde::Serializer::serialize_unit_variant(
                    _serializer,
                    #type_name,
                    #variant_index,
                    #variant_name,
                )
            }
        }
        Style::Newtype => {
            let field = &variant.fields[0];
            let mut field_expr = quote!(__field0);
            if let Some(path) = field.attrs.serialize_with() {
                field_expr = wrap_serialize_with(ident, generics, field.ty, path, field_expr);
            }

            quote_expr! {
                _serde::Serializer::serialize_newtype_variant(
                    _serializer,
                    #type_name,
                    #variant_index,
                    #variant_name,
                    #field_expr,
                )
            }
        }
        Style::Tuple => {
            serialize_tuple_variant(TupleVariant::ExternallyTagged {
                                        type_name: type_name,
                                        variant_index: variant_index,
                                        variant_name: variant_name,
                                    },
                                    ident,
                                    generics,
                                    &variant.fields)
        }
        Style::Struct => {
            serialize_struct_variant(StructVariant::ExternallyTagged {
                                         variant_index: variant_index,
                                         variant_name: variant_name,
                                     },
                                     ident,
                                     generics,
                                     &variant.fields,
                                     &type_name)
        }
    }
}

fn serialize_internally_tagged_variant(ident: &syn::Ident,
                                       generics: &syn::Generics,
                                       variant: &Variant,
                                       item_attrs: &attr::Item,
                                       tag: &str)
                                       -> Fragment {
    let type_name = item_attrs.name().serialize_name();
    let variant_name = variant.attrs.name().serialize_name();

    let enum_ident_str = ident.as_ref();
    let variant_ident_str = variant.ident.as_ref();

    match variant.style {
        Style::Unit => {
            quote_block! {
                let mut __struct = try!(_serde::Serializer::serialize_struct(
                    _serializer, #type_name, 1));
                try!(_serde::ser::SerializeStruct::serialize_field(
                    &mut __struct, #tag, #variant_name));
                _serde::ser::SerializeStruct::end(__struct)
            }
        }
        Style::Newtype => {
            let field = &variant.fields[0];
            let mut field_expr = quote!(__field0);
            if let Some(path) = field.attrs.serialize_with() {
                field_expr = wrap_serialize_with(ident, generics, field.ty, path, field_expr);
            }

            quote_expr! {
                _serde::ser::private::serialize_tagged_newtype(
                    _serializer,
                    #enum_ident_str,
                    #variant_ident_str,
                    #tag,
                    #variant_name,
                    #field_expr,
                )
            }
        }
        Style::Struct => {
            serialize_struct_variant(StructVariant::InternallyTagged {
                                         tag: tag,
                                         variant_name: variant_name,
                                     },
                                     ident,
                                     generics,
                                     &variant.fields,
                                     &type_name)
        }
        Style::Tuple => unreachable!("checked in serde_codegen_internals"),
    }
}

fn serialize_adjacently_tagged_variant(ident: &syn::Ident,
                                       generics: &syn::Generics,
                                       variant: &Variant,
                                       item_attrs: &attr::Item,
                                       tag: &str,
                                       content: &str)
                                       -> Fragment {
    let type_name = item_attrs.name().serialize_name();
    let variant_name = variant.attrs.name().serialize_name();

    let inner = Stmts(match variant.style {
        Style::Unit => {
            return quote_block! {
                let mut __struct = try!(_serde::Serializer::serialize_struct(
                    _serializer, #type_name, 1));
                try!(_serde::ser::SerializeStruct::serialize_field(
                    &mut __struct, #tag, #variant_name));
                _serde::ser::SerializeStruct::end(__struct)
            };
        }
        Style::Newtype => {
            let field = &variant.fields[0];
            let mut field_expr = quote!(__field0);
            if let Some(path) = field.attrs.serialize_with() {
                field_expr = wrap_serialize_with(ident, generics, field.ty, path, field_expr);
            }

            quote_expr! {
                _serde::Serialize::serialize(#field_expr, _serializer)
            }
        }
        Style::Tuple => {
            serialize_tuple_variant(TupleVariant::Untagged,
                                    ident,
                                    generics,
                                    &variant.fields)
        }
        Style::Struct => {
            serialize_struct_variant(StructVariant::Untagged,
                                     ident,
                                     generics,
                                     &variant.fields,
                                     &variant_name)
        }
    });

    let fields_ty = variant.fields.iter().map(|f| &f.ty);
    let ref fields_ident: Vec<_> = match variant.style {
        Style::Unit => unreachable!(),
        Style::Newtype => vec![Ident::new("__field0")],
        Style::Tuple => {
            (0..variant.fields.len())
                .map(|i| Ident::new(format!("__field{}", i)))
                .collect()
        }
        Style::Struct => {
            variant.fields
                .iter()
                .map(|f| f.ident.clone().expect("struct variant has unnamed fields"))
                .collect()
        }
    };

    let (_, ty_generics, where_clause) = generics.split_for_impl();

    let wrapper_generics = bound::with_lifetime_bound(generics, "'__a");
    let (wrapper_impl_generics, wrapper_ty_generics, _) = wrapper_generics.split_for_impl();

    quote_block! {
        struct __AdjacentlyTagged #wrapper_generics #where_clause {
            data: (#(&'__a #fields_ty,)*),
            phantom: _serde::export::PhantomData<#ident #ty_generics>,
        }

        impl #wrapper_impl_generics _serde::Serialize for __AdjacentlyTagged #wrapper_ty_generics #where_clause {
            fn serialize<__S>(&self, _serializer: __S) -> _serde::export::Result<__S::Ok, __S::Error>
                where __S: _serde::Serializer
            {
                let (#(#fields_ident,)*) = self.data;
                #inner
            }
        }

        let mut __struct = try!(_serde::Serializer::serialize_struct(
            _serializer, #type_name, 2));
        try!(_serde::ser::SerializeStruct::serialize_field(
            &mut __struct, #tag, #variant_name));
        try!(_serde::ser::SerializeStruct::serialize_field(
            &mut __struct, #content, &__AdjacentlyTagged {
                data: (#(#fields_ident,)*),
                phantom: _serde::export::PhantomData::<#ident #ty_generics>,
            }));
        _serde::ser::SerializeStruct::end(__struct)
    }
}

fn serialize_untagged_variant(ident: &syn::Ident,
                              generics: &syn::Generics,
                              variant: &Variant,
                              item_attrs: &attr::Item)
                              -> Fragment {
    match variant.style {
        Style::Unit => {
            quote_expr! {
                _serde::Serializer::serialize_unit(_serializer)
            }
        }
        Style::Newtype => {
            let field = &variant.fields[0];
            let mut field_expr = quote!(__field0);
            if let Some(path) = field.attrs.serialize_with() {
                field_expr = wrap_serialize_with(ident, generics, field.ty, path, field_expr);
            }

            quote_expr! {
                _serde::Serialize::serialize(#field_expr, _serializer)
            }
        }
        Style::Tuple => {
            serialize_tuple_variant(TupleVariant::Untagged, ident, generics, &variant.fields)
        }
        Style::Struct => {
            let type_name = item_attrs.name().serialize_name();
            serialize_struct_variant(StructVariant::Untagged,
                                     ident,
                                     generics,
                                     &variant.fields,
                                     &type_name)
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
                           ident: &syn::Ident,
                           generics: &syn::Generics,
                           fields: &[Field])
                           -> Fragment {
    let method = match context {
        TupleVariant::ExternallyTagged { .. } => {
            quote!(_serde::ser::SerializeTupleVariant::serialize_field)
        }
        TupleVariant::Untagged => quote!(_serde::ser::SerializeTuple::serialize_element),
    };

    let serialize_stmts =
        serialize_tuple_struct_visitor(ident, fields, generics, true, method);

    let len = serialize_stmts.len();
    let let_mut = mut_if(len > 0);

    match context {
        TupleVariant::ExternallyTagged { type_name, variant_index, variant_name } => {
            quote_block! {
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
            quote_block! {
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
                                ident: &syn::Ident,
                                generics: &syn::Generics,
                                fields: &[Field],
                                name: &str)
                                -> Fragment {
    let method = match context {
        StructVariant::ExternallyTagged { .. } => {
            quote!(_serde::ser::SerializeStructVariant::serialize_field)
        }
        StructVariant::InternallyTagged { .. } |
        StructVariant::Untagged => quote!(_serde::ser::SerializeStruct::serialize_field),
    };

    let serialize_fields = serialize_struct_visitor(ident, fields, generics, true, method);

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
            quote_block! {
                let #let_mut __serde_state = try!(_serde::Serializer::serialize_struct_variant(
                    _serializer,
                    #name,
                    #variant_index,
                    #variant_name,
                    #len,
                ));
                #(#serialize_fields)*
                _serde::ser::SerializeStructVariant::end(__serde_state)
            }
        }
        StructVariant::InternallyTagged { tag, variant_name } => {
            quote_block! {
                let mut __serde_state = try!(_serde::Serializer::serialize_struct(
                    _serializer,
                    #name,
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
            quote_block! {
                let #let_mut __serde_state = try!(_serde::Serializer::serialize_struct(
                    _serializer,
                    #name,
                    #len,
                ));
                #(#serialize_fields)*
                _serde::ser::SerializeStruct::end(__serde_state)
            }
        }
    }
}

fn serialize_tuple_struct_visitor(ident: &syn::Ident,
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
                    wrap_serialize_with(ident, generics, field.ty, path, field_expr);
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

fn serialize_struct_visitor(ident: &syn::Ident,
                            fields: &[Field],
                            generics: &syn::Generics,
                            is_enum: bool,
                            func: Tokens)
                            -> Vec<Tokens> {
    fields.iter()
        .filter(|&field| !field.attrs.skip_serializing())
        .map(|field| {
            let field_ident = field.ident.clone().expect("struct has unnamed field");
            let mut field_expr = if is_enum {
                quote!(#field_ident)
            } else {
                quote!(&self.#field_ident)
            };

            let key_expr = field.attrs.name().serialize_name();

            let skip = field.attrs
                .skip_serializing_if()
                .map(|path| quote!(#path(#field_expr)));

            if let Some(path) = field.attrs.serialize_with() {
                field_expr =
                    wrap_serialize_with(ident, generics, field.ty, path, field_expr)
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

fn wrap_serialize_with(ident: &syn::Ident,
                       generics: &syn::Generics,
                       field_ty: &syn::Ty,
                       serialize_with: &syn::Path,
                       value: Tokens)
                       -> Tokens {
    let (_, ty_generics, where_clause) = generics.split_for_impl();

    let wrapper_generics = bound::with_lifetime_bound(generics, "'__a");
    let (wrapper_impl_generics, wrapper_ty_generics, _) = wrapper_generics.split_for_impl();

    quote!({
        struct __SerializeWith #wrapper_impl_generics #where_clause {
            value: &'__a #field_ty,
            phantom: _serde::export::PhantomData<#ident #ty_generics>,
        }

        impl #wrapper_impl_generics _serde::Serialize for __SerializeWith #wrapper_ty_generics #where_clause {
            fn serialize<__S>(&self, __s: __S) -> _serde::export::Result<__S::Ok, __S::Error>
                where __S: _serde::Serializer
            {
                #serialize_with(self.value, __s)
            }
        }

        &__SerializeWith {
            value: #value,
            phantom: _serde::export::PhantomData::<#ident #ty_generics>,
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
