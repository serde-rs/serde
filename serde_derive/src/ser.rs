// Copyright 2017 Serde Developers
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use syn::{self, Ident};
use quote::Tokens;

use bound;
use fragment::{Fragment, Stmts, Match};
use internals::ast::{Body, Container, Field, Style, Variant};
use internals::{attr, Ctxt};

use std::u32;

pub fn expand_derive_serialize(input: &syn::DeriveInput) -> Result<Tokens, String> {
    let ctxt = Ctxt::new();
    let cont = Container::from_ast(&ctxt, input);
    precondition(&ctxt, &cont);
    try!(ctxt.check());

    let ident = &cont.ident;
    let params = Parameters::new(&cont);
    let (impl_generics, ty_generics, where_clause) = params.generics.split_for_impl();
    let dummy_const = Ident::new(format!("_IMPL_SERIALIZE_FOR_{}", ident));
    let body = Stmts(serialize_body(&cont, &params));

    let impl_block = if let Some(remote) = cont.attrs.remote() {
        quote! {
            impl #impl_generics #ident #ty_generics #where_clause {
                fn serialize<__S>(__self: &#remote #ty_generics, __serializer: __S) -> _serde::export::Result<__S::Ok, __S::Error>
                    where __S: _serde::Serializer
                {
                    #body
                }
            }
        }
    } else {
        quote! {
            #[automatically_derived]
            impl #impl_generics _serde::Serialize for #ident #ty_generics #where_clause {
                fn serialize<__S>(&self, __serializer: __S) -> _serde::export::Result<__S::Ok, __S::Error>
                    where __S: _serde::Serializer
                {
                    #body
                }
            }
        }
    };

    let generated = quote! {
        #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
        const #dummy_const: () = {
            extern crate serde as _serde;
            #impl_block
        };
    };
    Ok(generated)
}

fn precondition(cx: &Ctxt, cont: &Container) {
    match cont.attrs.identifier() {
        attr::Identifier::No => {}
        attr::Identifier::Field => {
            cx.error("field identifiers cannot be serialized");
        }
        attr::Identifier::Variant => {
            cx.error("variant identifiers cannot be serialized");
        }
    }
}

struct Parameters {
    /// Variable holding the value being serialized. Either `self` for local
    /// types or `__self` for remote types.
    self_var: Ident,

    /// Path to the type the impl is for. Either a single `Ident` for local
    /// types or `some::remote::Ident` for remote types. Does not include
    /// generic parameters.
    this: syn::Path,

    /// Generics including any explicit and inferred bounds for the impl.
    generics: syn::Generics,

    /// Type has a `serde(remote = "...")` attribute.
    is_remote: bool,
}

impl Parameters {
    fn new(cont: &Container) -> Self {
        let is_remote = cont.attrs.remote().is_some();
        let self_var = if is_remote {
            Ident::new("__self")
        } else {
            Ident::new("self")
        };

        let this = match cont.attrs.remote() {
            Some(remote) => remote.clone(),
            None => cont.ident.clone().into(),
        };

        let generics = build_generics(cont);

        Parameters {
            self_var: self_var,
            this: this,
            generics: generics,
            is_remote: is_remote,
        }
    }

    /// Type name to use in error messages and `&'static str` arguments to
    /// various Serializer methods.
    fn type_name(&self) -> &str {
        self.this.segments.last().unwrap().ident.as_ref()
    }
}

// All the generics in the input, plus a bound `T: Serialize` for each generic
// field type that will be serialized by us.
fn build_generics(cont: &Container) -> syn::Generics {
    let generics = bound::without_defaults(cont.generics);

    let generics =
        bound::with_where_predicates_from_fields(cont, &generics, attr::Field::ser_bound);

    match cont.attrs.ser_bound() {
        Some(predicates) => bound::with_where_predicates(&generics, predicates),
        None => {
            bound::with_bound(
                cont,
                &generics,
                needs_serialize_bound,
                &path!(_serde::Serialize),
            )
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

fn serialize_body(cont: &Container, params: &Parameters) -> Fragment {
    if let Some(into_type) = cont.attrs.into_type() {
        serialize_into(params, into_type)
    } else {
        match cont.body {
            Body::Enum(ref variants) => serialize_enum(params, variants, &cont.attrs),
            Body::Struct(Style::Struct, ref fields) => {
                if fields.iter().any(|field| field.ident.is_none()) {
                    panic!("struct has unnamed fields");
                }
                serialize_struct(params, fields, &cont.attrs)
            }
            Body::Struct(Style::Tuple, ref fields) => {
                if fields.iter().any(|field| field.ident.is_some()) {
                    panic!("tuple struct has named fields");
                }
                serialize_tuple_struct(params, fields, &cont.attrs)
            }
            Body::Struct(Style::Newtype, ref fields) => {
                serialize_newtype_struct(params, &fields[0], &cont.attrs)
            }
            Body::Struct(Style::Unit, _) => serialize_unit_struct(&cont.attrs),
        }
    }
}

fn serialize_into(params: &Parameters, into_type: &syn::Ty) -> Fragment {
    let self_var = &params.self_var;
    quote_block! {
        _serde::Serialize::serialize(
            &_serde::export::Into::<#into_type>::into(_serde::export::Clone::clone(#self_var)),
            __serializer)
    }
}

fn serialize_unit_struct(cattrs: &attr::Container) -> Fragment {
    let type_name = cattrs.name().serialize_name();

    quote_expr! {
        _serde::Serializer::serialize_unit_struct(__serializer, #type_name)
    }
}

fn serialize_newtype_struct(
    params: &Parameters,
    field: &Field,
    cattrs: &attr::Container,
) -> Fragment {
    let type_name = cattrs.name().serialize_name();

    let mut field_expr = get_field(params, field, 0);
    if let Some(path) = field.attrs.serialize_with() {
        field_expr = wrap_serialize_with(params, field.ty, path, field_expr);
    }

    quote_expr! {
        _serde::Serializer::serialize_newtype_struct(__serializer, #type_name, #field_expr)
    }
}

fn serialize_tuple_struct(
    params: &Parameters,
    fields: &[Field],
    cattrs: &attr::Container,
) -> Fragment {
    let serialize_stmts = serialize_tuple_struct_visitor(
        fields,
        params,
        false,
        quote!(_serde::ser::SerializeTupleStruct::serialize_field),
    );

    let type_name = cattrs.name().serialize_name();
    let len = serialize_stmts.len();
    let let_mut = mut_if(len > 0);

    quote_block! {
        let #let_mut __serde_state = try!(_serde::Serializer::serialize_tuple_struct(__serializer, #type_name, #len));
        #(#serialize_stmts)*
        _serde::ser::SerializeTupleStruct::end(__serde_state)
    }
}

fn serialize_struct(params: &Parameters, fields: &[Field], cattrs: &attr::Container) -> Fragment {
    assert!(fields.len() as u64 <= u32::MAX as u64);

    let serialize_fields = serialize_struct_visitor(
        fields,
        params,
        false,
        quote!(_serde::ser::SerializeStruct::serialize_field),
    );

    let type_name = cattrs.name().serialize_name();

    let mut serialized_fields = fields
        .iter()
        .filter(|&field| !field.attrs.skip_serializing())
        .peekable();

    let let_mut = mut_if(serialized_fields.peek().is_some());

    let len = serialized_fields
        .map(
            |field| match field.attrs.skip_serializing_if() {
                None => quote!(1),
                Some(path) => {
                    let ident = field.ident.clone().expect("struct has unnamed fields");
                    let field_expr = get_field(params, field, ident);
                    quote!(if #path(#field_expr) { 0 } else { 1 })
                }
            },
        )
        .fold(quote!(0), |sum, expr| quote!(#sum + #expr));

    quote_block! {
        let #let_mut __serde_state = try!(_serde::Serializer::serialize_struct(__serializer, #type_name, #len));
        #(#serialize_fields)*
        _serde::ser::SerializeStruct::end(__serde_state)
    }
}

fn serialize_enum(params: &Parameters, variants: &[Variant], cattrs: &attr::Container) -> Fragment {
    assert!(variants.len() as u64 <= u32::MAX as u64);

    let self_var = &params.self_var;

    let arms: Vec<_> = variants
        .iter()
        .enumerate()
        .map(
            |(variant_index, variant)| {
                serialize_variant(params, variant, variant_index as u32, cattrs)
            },
        )
        .collect();

    quote_expr! {
        match *#self_var {
            #(#arms)*
        }
    }
}

fn serialize_variant(
    params: &Parameters,
    variant: &Variant,
    variant_index: u32,
    cattrs: &attr::Container,
) -> Tokens {
    let this = &params.this;
    let variant_ident = variant.ident.clone();

    if variant.attrs.skip_serializing() {
        let skipped_msg = format!(
            "the enum variant {}::{} cannot be serialized",
            params.type_name(),
            variant_ident
        );
        let skipped_err = quote! {
            _serde::export::Err(_serde::ser::Error::custom(#skipped_msg))
        };
        let fields_pat = match variant.style {
            Style::Unit => quote!(),
            Style::Newtype | Style::Tuple => quote!((..)),
            Style::Struct => {
                quote!(
                    {
                        ..
                    }
                )
            }
        };
        quote! {
            #this::#variant_ident #fields_pat => #skipped_err,
        }
    } else {
        // variant wasn't skipped
        let case = match variant.style {
            Style::Unit => {
                quote! {
                    #this::#variant_ident
                }
            }
            Style::Newtype => {
                quote! {
                    #this::#variant_ident(ref __field0)
                }
            }
            Style::Tuple => {
                let field_names =
                    (0..variant.fields.len()).map(|i| Ident::new(format!("__field{}", i)));
                quote! {
                    #this::#variant_ident(#(ref #field_names),*)
                }
            }
            Style::Struct => {
                let fields = variant
                    .fields
                    .iter()
                    .map(
                        |f| {
                            f.ident
                                .clone()
                                .expect("struct variant has unnamed fields")
                        },
                    );
                quote! {
                    #this::#variant_ident { #(ref #fields),* }
                }
            }
        };

        let body = Match(
            match *cattrs.tag() {
                attr::EnumTag::External => {
                    serialize_externally_tagged_variant(params, variant, variant_index, cattrs)
                }
                attr::EnumTag::Internal { ref tag } => {
                    serialize_internally_tagged_variant(params, variant, cattrs, tag)
                }
                attr::EnumTag::Adjacent {
                    ref tag,
                    ref content,
                } => serialize_adjacently_tagged_variant(params, variant, cattrs, tag, content),
                attr::EnumTag::None => serialize_untagged_variant(params, variant, cattrs),
            },
        );

        quote! {
            #case => #body
        }
    }
}

fn serialize_externally_tagged_variant(
    params: &Parameters,
    variant: &Variant,
    variant_index: u32,
    cattrs: &attr::Container,
) -> Fragment {
    let type_name = cattrs.name().serialize_name();
    let variant_name = variant.attrs.name().serialize_name();

    match variant.style {
        Style::Unit => {
            quote_expr! {
                _serde::Serializer::serialize_unit_variant(
                    __serializer,
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
                field_expr = wrap_serialize_with(params, field.ty, path, field_expr);
            }

            quote_expr! {
                _serde::Serializer::serialize_newtype_variant(
                    __serializer,
                    #type_name,
                    #variant_index,
                    #variant_name,
                    #field_expr,
                )
            }
        }
        Style::Tuple => {
            serialize_tuple_variant(
                TupleVariant::ExternallyTagged {
                    type_name: type_name,
                    variant_index: variant_index,
                    variant_name: variant_name,
                },
                params,
                &variant.fields,
            )
        }
        Style::Struct => {
            serialize_struct_variant(
                StructVariant::ExternallyTagged {
                    variant_index: variant_index,
                    variant_name: variant_name,
                },
                params,
                &variant.fields,
                &type_name,
            )
        }
    }
}

fn serialize_internally_tagged_variant(
    params: &Parameters,
    variant: &Variant,
    cattrs: &attr::Container,
    tag: &str,
) -> Fragment {
    let type_name = cattrs.name().serialize_name();
    let variant_name = variant.attrs.name().serialize_name();

    let enum_ident_str = params.type_name();
    let variant_ident_str = variant.ident.as_ref();

    match variant.style {
        Style::Unit => {
            quote_block! {
                let mut __struct = try!(_serde::Serializer::serialize_struct(
                    __serializer, #type_name, 1));
                try!(_serde::ser::SerializeStruct::serialize_field(
                    &mut __struct, #tag, #variant_name));
                _serde::ser::SerializeStruct::end(__struct)
            }
        }
        Style::Newtype => {
            let field = &variant.fields[0];
            let mut field_expr = quote!(__field0);
            if let Some(path) = field.attrs.serialize_with() {
                field_expr = wrap_serialize_with(params, field.ty, path, field_expr);
            }

            quote_expr! {
                _serde::private::ser::serialize_tagged_newtype(
                    __serializer,
                    #enum_ident_str,
                    #variant_ident_str,
                    #tag,
                    #variant_name,
                    #field_expr,
                )
            }
        }
        Style::Struct => {
            serialize_struct_variant(
                StructVariant::InternallyTagged {
                    tag: tag,
                    variant_name: variant_name,
                },
                params,
                &variant.fields,
                &type_name,
            )
        }
        Style::Tuple => unreachable!("checked in serde_derive_internals"),
    }
}

fn serialize_adjacently_tagged_variant(
    params: &Parameters,
    variant: &Variant,
    cattrs: &attr::Container,
    tag: &str,
    content: &str,
) -> Fragment {
    let this = &params.this;
    let type_name = cattrs.name().serialize_name();
    let variant_name = variant.attrs.name().serialize_name();

    let inner = Stmts(
        match variant.style {
            Style::Unit => {
                return quote_block! {
                    let mut __struct = try!(_serde::Serializer::serialize_struct(
                        __serializer, #type_name, 1));
                    try!(_serde::ser::SerializeStruct::serialize_field(
                        &mut __struct, #tag, #variant_name));
                    _serde::ser::SerializeStruct::end(__struct)
                };
            }
            Style::Newtype => {
                let field = &variant.fields[0];
                let mut field_expr = quote!(__field0);
                if let Some(path) = field.attrs.serialize_with() {
                    field_expr = wrap_serialize_with(params, field.ty, path, field_expr);
                }

                quote_expr! {
                    _serde::Serialize::serialize(#field_expr, __serializer)
                }
            }
            Style::Tuple => {
                serialize_tuple_variant(TupleVariant::Untagged, params, &variant.fields)
            }
            Style::Struct => {
                serialize_struct_variant(
                    StructVariant::Untagged,
                    params,
                    &variant.fields,
                    &variant_name,
                )
            }
        },
    );

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
            variant
                .fields
                .iter()
                .map(
                    |f| {
                        f.ident
                            .clone()
                            .expect("struct variant has unnamed fields")
                    },
                )
                .collect()
        }
    };

    let (_, ty_generics, where_clause) = params.generics.split_for_impl();

    let wrapper_generics = bound::with_lifetime_bound(&params.generics, "'__a");
    let (wrapper_impl_generics, wrapper_ty_generics, _) = wrapper_generics.split_for_impl();

    quote_block! {
        struct __AdjacentlyTagged #wrapper_generics #where_clause {
            data: (#(&'__a #fields_ty,)*),
            phantom: _serde::export::PhantomData<#this #ty_generics>,
        }

        impl #wrapper_impl_generics _serde::Serialize for __AdjacentlyTagged #wrapper_ty_generics #where_clause {
            fn serialize<__S>(&self, __serializer: __S) -> _serde::export::Result<__S::Ok, __S::Error>
                where __S: _serde::Serializer
            {
                let (#(#fields_ident,)*) = self.data;
                #inner
            }
        }

        let mut __struct = try!(_serde::Serializer::serialize_struct(
            __serializer, #type_name, 2));
        try!(_serde::ser::SerializeStruct::serialize_field(
            &mut __struct, #tag, #variant_name));
        try!(_serde::ser::SerializeStruct::serialize_field(
            &mut __struct, #content, &__AdjacentlyTagged {
                data: (#(#fields_ident,)*),
                phantom: _serde::export::PhantomData::<#this #ty_generics>,
            }));
        _serde::ser::SerializeStruct::end(__struct)
    }
}

fn serialize_untagged_variant(
    params: &Parameters,
    variant: &Variant,
    cattrs: &attr::Container,
) -> Fragment {
    match variant.style {
        Style::Unit => {
            quote_expr! {
                _serde::Serializer::serialize_unit(__serializer)
            }
        }
        Style::Newtype => {
            let field = &variant.fields[0];
            let mut field_expr = quote!(__field0);
            if let Some(path) = field.attrs.serialize_with() {
                field_expr = wrap_serialize_with(params, field.ty, path, field_expr);
            }

            quote_expr! {
                _serde::Serialize::serialize(#field_expr, __serializer)
            }
        }
        Style::Tuple => serialize_tuple_variant(TupleVariant::Untagged, params, &variant.fields),
        Style::Struct => {
            let type_name = cattrs.name().serialize_name();
            serialize_struct_variant(StructVariant::Untagged, params, &variant.fields, &type_name)
        }
    }
}

enum TupleVariant {
    ExternallyTagged {
        type_name: String,
        variant_index: u32,
        variant_name: String,
    },
    Untagged,
}

fn serialize_tuple_variant(
    context: TupleVariant,
    params: &Parameters,
    fields: &[Field],
) -> Fragment {
    let method = match context {
        TupleVariant::ExternallyTagged { .. } => {
            quote!(_serde::ser::SerializeTupleVariant::serialize_field)
        }
        TupleVariant::Untagged => quote!(_serde::ser::SerializeTuple::serialize_element),
    };

    let serialize_stmts = serialize_tuple_struct_visitor(fields, params, true, method);

    let len = serialize_stmts.len();
    let let_mut = mut_if(len > 0);

    match context {
        TupleVariant::ExternallyTagged {
            type_name,
            variant_index,
            variant_name,
        } => {
            quote_block! {
                let #let_mut __serde_state = try!(_serde::Serializer::serialize_tuple_variant(
                    __serializer,
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
                    __serializer,
                    #len));
                #(#serialize_stmts)*
                _serde::ser::SerializeTuple::end(__serde_state)
            }
        }
    }
}

enum StructVariant<'a> {
    ExternallyTagged {
        variant_index: u32,
        variant_name: String,
    },
    InternallyTagged { tag: &'a str, variant_name: String },
    Untagged,
}

fn serialize_struct_variant<'a>(
    context: StructVariant<'a>,
    params: &Parameters,
    fields: &[Field],
    name: &str,
) -> Fragment {
    let method = match context {
        StructVariant::ExternallyTagged { .. } => {
            quote!(_serde::ser::SerializeStructVariant::serialize_field)
        }
        StructVariant::InternallyTagged { .. } |
        StructVariant::Untagged => quote!(_serde::ser::SerializeStruct::serialize_field),
    };

    let serialize_fields = serialize_struct_visitor(fields, params, true, method);

    let mut serialized_fields = fields
        .iter()
        .filter(|&field| !field.attrs.skip_serializing())
        .peekable();

    let let_mut = mut_if(serialized_fields.peek().is_some());

    let len = serialized_fields
        .map(
            |field| {
                let ident = field.ident.clone().expect("struct has unnamed fields");

                match field.attrs.skip_serializing_if() {
                    Some(path) => quote!(if #path(#ident) { 0 } else { 1 }),
                    None => quote!(1),
                }
            },
        )
        .fold(quote!(0), |sum, expr| quote!(#sum + #expr));

    match context {
        StructVariant::ExternallyTagged {
            variant_index,
            variant_name,
        } => {
            quote_block! {
                let #let_mut __serde_state = try!(_serde::Serializer::serialize_struct_variant(
                    __serializer,
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
                    __serializer,
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
                    __serializer,
                    #name,
                    #len,
                ));
                #(#serialize_fields)*
                _serde::ser::SerializeStruct::end(__serde_state)
            }
        }
    }
}

fn serialize_tuple_struct_visitor(
    fields: &[Field],
    params: &Parameters,
    is_enum: bool,
    func: Tokens,
) -> Vec<Tokens> {
    fields
        .iter()
        .enumerate()
        .map(
            |(i, field)| {
                let mut field_expr = if is_enum {
                    let id = Ident::new(format!("__field{}", i));
                    quote!(#id)
                } else {
                    get_field(params, field, i)
                };

                let skip = field
                    .attrs
                    .skip_serializing_if()
                    .map(|path| quote!(#path(#field_expr)));

                if let Some(path) = field.attrs.serialize_with() {
                    field_expr = wrap_serialize_with(params, field.ty, path, field_expr);
                }

                let ser = quote! {
                    try!(#func(&mut __serde_state, #field_expr));
                };

                match skip {
                    None => ser,
                    Some(skip) => quote!(if !#skip { #ser }),
                }
            },
        )
        .collect()
}

fn serialize_struct_visitor(
    fields: &[Field],
    params: &Parameters,
    is_enum: bool,
    func: Tokens,
) -> Vec<Tokens> {
    fields
        .iter()
        .filter(|&field| !field.attrs.skip_serializing())
        .map(
            |field| {
                let field_ident = field.ident.clone().expect("struct has unnamed field");
                let mut field_expr = if is_enum {
                    quote!(#field_ident)
                } else {
                    get_field(params, field, field_ident)
                };

                let key_expr = field.attrs.name().serialize_name();

                let skip = field
                    .attrs
                    .skip_serializing_if()
                    .map(|path| quote!(#path(#field_expr)));

                if let Some(path) = field.attrs.serialize_with() {
                    field_expr = wrap_serialize_with(params, field.ty, path, field_expr)
                }

                let ser = quote! {
                    try!(#func(&mut __serde_state, #key_expr, #field_expr));
                };

                match skip {
                    None => ser,
                    Some(skip) => quote!(if !#skip { #ser }),
                }
            },
        )
        .collect()
}

fn wrap_serialize_with(
    params: &Parameters,
    field_ty: &syn::Ty,
    serialize_with: &syn::Path,
    value: Tokens,
) -> Tokens {
    let this = &params.this;
    let (_, ty_generics, where_clause) = params.generics.split_for_impl();

    let wrapper_generics = bound::with_lifetime_bound(&params.generics, "'__a");
    let (wrapper_impl_generics, wrapper_ty_generics, _) = wrapper_generics.split_for_impl();

    quote!({
        struct __SerializeWith #wrapper_impl_generics #where_clause {
            value: &'__a #field_ty,
            phantom: _serde::export::PhantomData<#this #ty_generics>,
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
            phantom: _serde::export::PhantomData::<#this #ty_generics>,
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

fn get_field<I>(params: &Parameters, field: &Field, ident: I) -> Tokens
where
    I: Into<Ident>,
{
    let self_var = &params.self_var;
    match (params.is_remote, field.attrs.getter()) {
        (false, None) => {
            let ident = ident.into();
            quote!(&#self_var.#ident)
        }
        (true, None) => {
            let ty = field.ty;
            let ident = ident.into();
            quote!(_serde::private::ser::constrain::<#ty>(&#self_var.#ident))
        }
        (true, Some(getter)) => {
            let ty = field.ty;
            quote!(_serde::private::ser::constrain::<#ty>(&#getter(#self_var)))
        }
        (false, Some(_)) => {
            unreachable!("getter is only allowed for remote impls");
        }
    }
}
