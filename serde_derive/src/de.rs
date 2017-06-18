// Copyright 2017 Serde Developers
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use syn::{self, Ident};
use quote::{self, Tokens, ToTokens};

use bound;
use fragment::{Fragment, Expr, Stmts, Match};
use internals::ast::{Body, Container, Field, Style, Variant};
use internals::{self, attr};

use std::collections::BTreeSet;

pub fn expand_derive_deserialize(input: &syn::DeriveInput) -> Result<Tokens, String> {
    let ctxt = internals::Ctxt::new();
    let cont = Container::from_ast(&ctxt, input);
    try!(ctxt.check());

    let ident = &cont.ident;
    let params = Parameters::new(&cont);
    let dummy_const = Ident::new(format!("_IMPL_DESERIALIZE_FOR_{}", ident));
    let body = Stmts(deserialize_body(&cont, &params));

    let impl_block = if let Some(remote) = cont.attrs.remote() {
        let (impl_generics, ty_generics, where_clause) = cont.generics.split_for_impl();
        let de_lifetime = params.de_lifetime_def();
        quote! {
            impl #impl_generics #ident #ty_generics #where_clause {
                fn deserialize<#de_lifetime, __D>(__deserializer: __D) -> _serde::export::Result<#remote #ty_generics, __D::Error>
                    where __D: _serde::Deserializer<'de>
                {
                    #body
                }
            }
        }
    } else {
        let (de_impl_generics, _, ty_generics, where_clause) = split_with_de_lifetime(&params);
        quote! {
            #[automatically_derived]
            impl #de_impl_generics _serde::Deserialize<'de> for #ident #ty_generics #where_clause {
                fn deserialize<__D>(__deserializer: __D) -> _serde::export::Result<Self, __D::Error>
                    where __D: _serde::Deserializer<'de>
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

struct Parameters {
    /// Name of the type the `derive` is on.
    local: syn::Ident,

    /// Path to the type the impl is for. Either a single `Ident` for local
    /// types or `some::remote::Ident` for remote types. Does not include
    /// generic parameters.
    this: syn::Path,

    /// Generics including any explicit and inferred bounds for the impl.
    generics: syn::Generics,

    /// Lifetimes borrowed from the deserializer. These will become bounds on
    /// the `'de` lifetime of the deserializer.
    borrowed: BTreeSet<syn::Lifetime>,

    /// At least one field has a serde(getter) attribute, implying that the
    /// remote type has a private field.
    has_getter: bool,
}

impl Parameters {
    fn new(cont: &Container) -> Self {
        let local = cont.ident.clone();
        let this = match cont.attrs.remote() {
            Some(remote) => remote.clone(),
            None => cont.ident.clone().into(),
        };
        let generics = build_generics(cont);
        let borrowed = borrowed_lifetimes(cont);
        let has_getter = cont.body.has_getter();

        Parameters {
            local: local,
            this: this,
            generics: generics,
            borrowed: borrowed,
            has_getter: has_getter,
        }
    }

    /// Type name to use in error messages and `&'static str` arguments to
    /// various Deserializer methods.
    fn type_name(&self) -> &str {
        self.this.segments.last().unwrap().ident.as_ref()
    }

    fn de_lifetime_def(&self) -> syn::LifetimeDef {
        syn::LifetimeDef {
            attrs: Vec::new(),
            lifetime: syn::Lifetime::new("'de"),
            bounds: self.borrowed.iter().cloned().collect(),
        }
    }
}

// All the generics in the input, plus a bound `T: Deserialize` for each generic
// field type that will be deserialized by us, plus a bound `T: Default` for
// each generic field type that will be set to a default value.
fn build_generics(cont: &Container) -> syn::Generics {
    let generics = bound::without_defaults(cont.generics);

    let generics = bound::with_where_predicates_from_fields(cont, &generics, attr::Field::de_bound);

    match cont.attrs.de_bound() {
        Some(predicates) => bound::with_where_predicates(&generics, predicates),
        None => {
            let generics = match *cont.attrs.default() {
                attr::Default::Default => {
                    bound::with_self_bound(cont, &generics, &path!(_serde::export::Default))
                }
                attr::Default::None |
                attr::Default::Path(_) => generics,
            };

            let generics = bound::with_bound(
                cont,
                &generics,
                needs_deserialize_bound,
                &path!(_serde::Deserialize<'de>),
            );

            bound::with_bound(
                cont,
                &generics,
                requires_default,
                &path!(_serde::export::Default),
            )
        }
    }
}

// Fields with a `skip_deserializing` or `deserialize_with` attribute are not
// deserialized by us so we do not generate a bound. Fields with a `bound`
// attribute specify their own bound so we do not generate one. All other fields
// may need a `T: Deserialize` bound where T is the type of the field.
fn needs_deserialize_bound(attrs: &attr::Field) -> bool {
    !attrs.skip_deserializing() && attrs.deserialize_with().is_none() && attrs.de_bound().is_none()
}

// Fields with a `default` attribute (not `default=...`), and fields with a
// `skip_deserializing` attribute that do not also have `default=...`.
fn requires_default(attrs: &attr::Field) -> bool {
    attrs.default() == &attr::Default::Default
}

// The union of lifetimes borrowed by each field of the container.
//
// These turn into bounds on the `'de` lifetime of the Deserialize impl. If
// lifetimes `'a` and `'b` are borrowed but `'c` is not, the impl is:
//
//     impl<'de: 'a + 'b, 'a, 'b, 'c> Deserialize<'de> for S<'a, 'b, 'c>
fn borrowed_lifetimes(cont: &Container) -> BTreeSet<syn::Lifetime> {
    let mut lifetimes = BTreeSet::new();
    for field in cont.body.all_fields() {
        lifetimes.extend(field.attrs.borrowed_lifetimes().iter().cloned());
    }
    lifetimes
}

fn deserialize_body(cont: &Container, params: &Parameters) -> Fragment {
    if let Some(from_type) = cont.attrs.from_type() {
        deserialize_from(from_type)
    } else if let attr::Identifier::No = cont.attrs.identifier() {
        match cont.body {
            Body::Enum(ref variants) => deserialize_enum(params, variants, &cont.attrs),
            Body::Struct(Style::Struct, ref fields) => {
                if fields.iter().any(|field| field.ident.is_none()) {
                    panic!("struct has unnamed fields");
                }
                deserialize_struct(None, params, fields, &cont.attrs, None)
            }
            Body::Struct(Style::Tuple, ref fields) |
            Body::Struct(Style::Newtype, ref fields) => {
                if fields.iter().any(|field| field.ident.is_some()) {
                    panic!("tuple struct has named fields");
                }
                deserialize_tuple(None, params, fields, &cont.attrs, None)
            }
            Body::Struct(Style::Unit, _) => deserialize_unit_struct(params, &cont.attrs),
        }
    } else {
        match cont.body {
            Body::Enum(ref variants) => {
                deserialize_custom_identifier(params, variants, &cont.attrs)
            }
            Body::Struct(_, _) => unreachable!("checked in serde_derive_internals"),
        }
    }
}

fn deserialize_from(from_type: &syn::Ty) -> Fragment {
    quote_block! {
        _serde::export::Result::map(
            <#from_type as _serde::Deserialize>::deserialize(__deserializer),
            _serde::export::From::from)
    }
}

fn deserialize_unit_struct(params: &Parameters, cattrs: &attr::Container) -> Fragment {
    let this = &params.this;
    let type_name = cattrs.name().deserialize_name();

    let expecting = format!("unit struct {}", params.type_name());

    quote_block! {
        struct __Visitor;

        impl<'de> _serde::de::Visitor<'de> for __Visitor {
            type Value = #this;

            fn expecting(&self, formatter: &mut _serde::export::Formatter) -> _serde::export::fmt::Result {
                _serde::export::Formatter::write_str(formatter, #expecting)
            }

            #[inline]
            fn visit_unit<__E>(self) -> _serde::export::Result<Self::Value, __E>
                where __E: _serde::de::Error
            {
                _serde::export::Ok(#this)
            }
        }

        _serde::Deserializer::deserialize_unit_struct(__deserializer, #type_name, __Visitor)
    }
}

fn deserialize_tuple(
    variant_ident: Option<&syn::Ident>,
    params: &Parameters,
    fields: &[Field],
    cattrs: &attr::Container,
    deserializer: Option<Tokens>,
) -> Fragment {
    let this = &params.this;
    let (de_impl_generics, de_ty_generics, ty_generics, where_clause) = split_with_de_lifetime(params,);

    // If there are getters (implying private fields), construct the local type
    // and use an `Into` conversion to get the remote type. If there are no
    // getters then construct the target type directly.
    let construct = if params.has_getter {
        let local = &params.local;
        quote!(#local)
    } else {
        quote!(#this)
    };

    let is_enum = variant_ident.is_some();
    let type_path = match variant_ident {
        Some(variant_ident) => quote!(#construct::#variant_ident),
        None => construct,
    };
    let expecting = match variant_ident {
        Some(variant_ident) => format!("tuple variant {}::{}", params.type_name(), variant_ident),
        None => format!("tuple struct {}", params.type_name()),
    };

    let nfields = fields.len();

    let visit_newtype_struct = if !is_enum && nfields == 1 {
        Some(deserialize_newtype_struct(&type_path, params, &fields[0]))
    } else {
        None
    };

    let visit_seq = Stmts(deserialize_seq(&type_path, params, fields, false, cattrs));

    let visitor_expr = quote! {
        __Visitor {
            marker: _serde::export::PhantomData::<#this #ty_generics>,
            lifetime: _serde::export::PhantomData,
        }
    };
    let dispatch = if let Some(deserializer) = deserializer {
        quote!(_serde::Deserializer::deserialize_tuple(#deserializer, #nfields, #visitor_expr))
    } else if is_enum {
        quote!(_serde::de::VariantAccess::tuple_variant(__variant, #nfields, #visitor_expr))
    } else if nfields == 1 {
        let type_name = cattrs.name().deserialize_name();
        quote!(_serde::Deserializer::deserialize_newtype_struct(__deserializer, #type_name, #visitor_expr))
    } else {
        let type_name = cattrs.name().deserialize_name();
        quote!(_serde::Deserializer::deserialize_tuple_struct(__deserializer, #type_name, #nfields, #visitor_expr))
    };

    let all_skipped = fields
        .iter()
        .all(|field| field.attrs.skip_deserializing());
    let visitor_var = if all_skipped {
        quote!(_)
    } else {
        quote!(mut __seq)
    };

    quote_block! {
        struct __Visitor #de_impl_generics #where_clause {
            marker: _serde::export::PhantomData<#this #ty_generics>,
            lifetime: _serde::export::PhantomData<&'de ()>,
        }

        impl #de_impl_generics _serde::de::Visitor<'de> for __Visitor #de_ty_generics #where_clause {
            type Value = #this #ty_generics;

            fn expecting(&self, formatter: &mut _serde::export::Formatter) -> _serde::export::fmt::Result {
                _serde::export::Formatter::write_str(formatter, #expecting)
            }

            #visit_newtype_struct

            #[inline]
            fn visit_seq<__A>(self, #visitor_var: __A) -> _serde::export::Result<Self::Value, __A::Error>
                where __A: _serde::de::SeqAccess<'de>
            {
                #visit_seq
            }
        }

        #dispatch
    }
}

fn deserialize_seq(
    type_path: &Tokens,
    params: &Parameters,
    fields: &[Field],
    is_struct: bool,
    cattrs: &attr::Container,
) -> Fragment {
    let vars = (0..fields.len()).map(field_i as fn(_) -> _);

    let deserialized_count = fields
        .iter()
        .filter(|field| !field.attrs.skip_deserializing())
        .count();
    let expecting = format!("tuple of {} elements", deserialized_count);

    let mut index_in_seq = 0usize;
    let let_values = vars.clone().zip(fields)
        .map(|(var, field)| {
            if field.attrs.skip_deserializing() {
                let default = Expr(expr_is_missing(&field, cattrs));
                quote! {
                    let #var = #default;
                }
            } else {
                let visit = match field.attrs.deserialize_with() {
                    None => {
                        let field_ty = &field.ty;
                        quote!(try!(_serde::de::SeqAccess::next_element::<#field_ty>(&mut __seq)))
                    }
                    Some(path) => {
                        let (wrapper, wrapper_ty) = wrap_deserialize_with(
                            params, field.ty, path);
                        quote!({
                            #wrapper
                            _serde::export::Option::map(
                                try!(_serde::de::SeqAccess::next_element::<#wrapper_ty>(&mut __seq)),
                                |__wrap| __wrap.value)
                        })
                    }
                };
                let assign = quote! {
                    let #var = match #visit {
                        Some(__value) => __value,
                        None => {
                            return _serde::export::Err(_serde::de::Error::invalid_length(#index_in_seq, &#expecting));
                        }
                    };
                };
                index_in_seq += 1;
                assign
            }
        });

    let mut result = if is_struct {
        let names = fields.iter().map(|f| &f.ident);
        quote! {
            #type_path { #( #names: #vars ),* }
        }
    } else {
        quote! {
            #type_path ( #(#vars),* )
        }
    };

    if params.has_getter {
        let this = &params.this;
        result = quote! {
            _serde::export::Into::<#this>::into(#result)
        };
    }

    quote_block! {
        #(#let_values)*
        _serde::export::Ok(#result)
    }
}

fn deserialize_newtype_struct(type_path: &Tokens, params: &Parameters, field: &Field) -> Tokens {
    let value = match field.attrs.deserialize_with() {
        None => {
            let field_ty = &field.ty;
            quote! {
                try!(<#field_ty as _serde::Deserialize>::deserialize(__e))
            }
        }
        Some(path) => {
            let (wrapper, wrapper_ty) = wrap_deserialize_with(params, field.ty, path);
            quote!({
                #wrapper
                try!(<#wrapper_ty as _serde::Deserialize>::deserialize(__e)).value
            })
        }
    };

    let mut result = quote!(#type_path(#value));
    if params.has_getter {
        let this = &params.this;
        result = quote! {
            _serde::export::Into::<#this>::into(#result)
        };
    }

    quote! {
        #[inline]
        fn visit_newtype_struct<__E>(self, __e: __E) -> _serde::export::Result<Self::Value, __E::Error>
            where __E: _serde::Deserializer<'de>
        {
            _serde::export::Ok(#result)
        }
    }
}

fn deserialize_struct(
    variant_ident: Option<&syn::Ident>,
    params: &Parameters,
    fields: &[Field],
    cattrs: &attr::Container,
    deserializer: Option<Tokens>,
) -> Fragment {
    let is_enum = variant_ident.is_some();
    let is_untagged = deserializer.is_some();

    let this = &params.this;
    let (de_impl_generics, de_ty_generics, ty_generics, where_clause) = split_with_de_lifetime(params,);

    // If there are getters (implying private fields), construct the local type
    // and use an `Into` conversion to get the remote type. If there are no
    // getters then construct the target type directly.
    let construct = if params.has_getter {
        let local = &params.local;
        quote!(#local)
    } else {
        quote!(#this)
    };

    let type_path = match variant_ident {
        Some(variant_ident) => quote!(#construct::#variant_ident),
        None => construct,
    };
    let expecting = match variant_ident {
        Some(variant_ident) => format!("struct variant {}::{}", params.type_name(), variant_ident),
        None => format!("struct {}", params.type_name()),
    };

    let visit_seq = Stmts(deserialize_seq(&type_path, params, fields, true, cattrs));

    let (field_visitor, fields_stmt, visit_map) =
        deserialize_struct_visitor(type_path, params, fields, cattrs);
    let field_visitor = Stmts(field_visitor);
    let fields_stmt = Stmts(fields_stmt);
    let visit_map = Stmts(visit_map);

    let visitor_expr = quote! {
        __Visitor {
            marker: _serde::export::PhantomData::<#this #ty_generics>,
            lifetime: _serde::export::PhantomData,
        }
    };
    let dispatch = if let Some(deserializer) = deserializer {
        quote! {
            _serde::Deserializer::deserialize_any(#deserializer, #visitor_expr)
        }
    } else if is_enum {
        quote! {
            _serde::de::VariantAccess::struct_variant(__variant, FIELDS, #visitor_expr)
        }
    } else {
        let type_name = cattrs.name().deserialize_name();
        quote! {
            _serde::Deserializer::deserialize_struct(__deserializer, #type_name, FIELDS, #visitor_expr)
        }
    };

    let all_skipped = fields
        .iter()
        .all(|field| field.attrs.skip_deserializing());
    let visitor_var = if all_skipped {
        quote!(_)
    } else {
        quote!(mut __seq)
    };

    let visit_seq = if is_untagged {
        // untagged struct variants do not get a visit_seq method
        None
    } else {
        Some(quote! {
            #[inline]
            fn visit_seq<__A>(self, #visitor_var: __A) -> _serde::export::Result<Self::Value, __A::Error>
                where __A: _serde::de::SeqAccess<'de>
            {
                #visit_seq
            }
        })
    };

    quote_block! {
        #field_visitor

        struct __Visitor #de_impl_generics #where_clause {
            marker: _serde::export::PhantomData<#this #ty_generics>,
            lifetime: _serde::export::PhantomData<&'de ()>,
        }

        impl #de_impl_generics _serde::de::Visitor<'de> for __Visitor #de_ty_generics #where_clause {
            type Value = #this #ty_generics;

            fn expecting(&self, formatter: &mut _serde::export::Formatter) -> _serde::export::fmt::Result {
                _serde::export::Formatter::write_str(formatter, #expecting)
            }

            #visit_seq

            #[inline]
            fn visit_map<__A>(self, mut __map: __A) -> _serde::export::Result<Self::Value, __A::Error>
                where __A: _serde::de::MapAccess<'de>
            {
                #visit_map
            }
        }

        #fields_stmt

        #dispatch
    }
}

fn deserialize_enum(
    params: &Parameters,
    variants: &[Variant],
    cattrs: &attr::Container,
) -> Fragment {
    match *cattrs.tag() {
        attr::EnumTag::External => deserialize_externally_tagged_enum(params, variants, cattrs),
        attr::EnumTag::Internal { ref tag } => {
            deserialize_internally_tagged_enum(params, variants, cattrs, tag)
        }
        attr::EnumTag::Adjacent {
            ref tag,
            ref content,
        } => deserialize_adjacently_tagged_enum(params, variants, cattrs, tag, content),
        attr::EnumTag::None => deserialize_untagged_enum(params, variants, cattrs),
    }
}

fn deserialize_externally_tagged_enum(
    params: &Parameters,
    variants: &[Variant],
    cattrs: &attr::Container,
) -> Fragment {
    let this = &params.this;
    let (de_impl_generics, de_ty_generics, ty_generics, where_clause) = split_with_de_lifetime(params,);

    let type_name = cattrs.name().deserialize_name();

    let expecting = format!("enum {}", params.type_name());

    let variant_names_idents: Vec<_> = variants
        .iter()
        .enumerate()
        .filter(|&(_, variant)| !variant.attrs.skip_deserializing())
        .map(|(i, variant)| (variant.attrs.name().deserialize_name(), field_i(i)),)
        .collect();

    let variants_stmt = {
        let variant_names = variant_names_idents.iter().map(|&(ref name, _)| name);
        quote! {
            const VARIANTS: &'static [&'static str] = &[ #(#variant_names),* ];
        }
    };

    let variant_visitor = Stmts(deserialize_generated_identifier(variant_names_idents, cattrs, true),);

    // Match arms to extract a variant from a string
    let variant_arms = variants
        .iter()
        .enumerate()
        .filter(|&(_, variant)| !variant.attrs.skip_deserializing())
        .map(
            |(i, variant)| {
                let variant_name = field_i(i);

                let block = Match(deserialize_externally_tagged_variant(params, variant, cattrs),);

                quote! {
                    (__Field::#variant_name, __variant) => #block
                }
            },
        );

    let all_skipped = variants
        .iter()
        .all(|variant| variant.attrs.skip_deserializing());
    let match_variant = if all_skipped {
        // This is an empty enum like `enum Impossible {}` or an enum in which
        // all variants have `#[serde(skip_deserializing)]`.
        quote! {
            // FIXME: Once we drop support for Rust 1.15:
            // let _serde::export::Err(__err) = _serde::de::EnumAccess::variant::<__Field>(__data);
            // _serde::export::Err(__err)
            _serde::export::Result::map(
                _serde::de::EnumAccess::variant::<__Field>(__data),
                |(__impossible, _)| match __impossible {})
        }
    } else {
        quote! {
            match try!(_serde::de::EnumAccess::variant(__data)) {
                #(#variant_arms)*
            }
        }
    };

    quote_block! {
        #variant_visitor

        struct __Visitor #de_impl_generics #where_clause {
            marker: _serde::export::PhantomData<#this #ty_generics>,
            lifetime: _serde::export::PhantomData<&'de ()>,
        }

        impl #de_impl_generics _serde::de::Visitor<'de> for __Visitor #de_ty_generics #where_clause {
            type Value = #this #ty_generics;

            fn expecting(&self, formatter: &mut _serde::export::Formatter) -> _serde::export::fmt::Result {
                _serde::export::Formatter::write_str(formatter, #expecting)
            }

            fn visit_enum<__A>(self, __data: __A) -> _serde::export::Result<Self::Value, __A::Error>
                where __A: _serde::de::EnumAccess<'de>
            {
                #match_variant
            }
        }

        #variants_stmt

        _serde::Deserializer::deserialize_enum(__deserializer, #type_name, VARIANTS,
                                               __Visitor {
                                                   marker: _serde::export::PhantomData::<#this #ty_generics>,
                                                   lifetime: _serde::export::PhantomData,
                                               })
    }
}

fn deserialize_internally_tagged_enum(
    params: &Parameters,
    variants: &[Variant],
    cattrs: &attr::Container,
    tag: &str,
) -> Fragment {
    let variant_names_idents: Vec<_> = variants
        .iter()
        .enumerate()
        .filter(|&(_, variant)| !variant.attrs.skip_deserializing())
        .map(|(i, variant)| (variant.attrs.name().deserialize_name(), field_i(i)),)
        .collect();

    let variants_stmt = {
        let variant_names = variant_names_idents.iter().map(|&(ref name, _)| name);
        quote! {
            const VARIANTS: &'static [&'static str] = &[ #(#variant_names),* ];
        }
    };

    let variant_visitor = Stmts(deserialize_generated_identifier(variant_names_idents, cattrs, true),);

    // Match arms to extract a variant from a string
    let variant_arms = variants.iter()
        .enumerate()
        .filter(|&(_, variant)| !variant.attrs.skip_deserializing())
        .map(|(i, variant)| {
            let variant_name = field_i(i);

            let block = Match(deserialize_internally_tagged_variant(
                params,
                variant,
                cattrs,
                quote!(_serde::private::de::ContentDeserializer::<__D::Error>::new(__tagged.content)),
            ));

            quote! {
                __Field::#variant_name => #block
            }
        });

    quote_block! {
        #variant_visitor

        #variants_stmt

        let __tagged = try!(_serde::Deserializer::deserialize_any(
            __deserializer,
            _serde::private::de::TaggedContentVisitor::<__Field>::new(#tag)));

        match __tagged.tag {
            #(#variant_arms)*
        }
    }
}

fn deserialize_adjacently_tagged_enum(
    params: &Parameters,
    variants: &[Variant],
    cattrs: &attr::Container,
    tag: &str,
    content: &str,
) -> Fragment {
    let this = &params.this;
    let (de_impl_generics, de_ty_generics, ty_generics, where_clause) = split_with_de_lifetime(params,);

    let variant_names_idents: Vec<_> = variants
        .iter()
        .enumerate()
        .filter(|&(_, variant)| !variant.attrs.skip_deserializing())
        .map(|(i, variant)| (variant.attrs.name().deserialize_name(), field_i(i)),)
        .collect();

    let variants_stmt = {
        let variant_names = variant_names_idents.iter().map(|&(ref name, _)| name);
        quote! {
            const VARIANTS: &'static [&'static str] = &[ #(#variant_names),* ];
        }
    };

    let variant_visitor = Stmts(deserialize_generated_identifier(variant_names_idents, cattrs, true),);

    let ref variant_arms: Vec<_> = variants
        .iter()
        .enumerate()
        .filter(|&(_, variant)| !variant.attrs.skip_deserializing())
        .map(
            |(i, variant)| {
                let variant_index = field_i(i);

                let block = Match(
                    deserialize_untagged_variant(
                        params,
                        variant,
                        cattrs,
                        quote!(__deserializer),
                    ),
                );

                quote! {
                    __Field::#variant_index => #block
                }
            },
        )
        .collect();

    let expecting = format!("adjacently tagged enum {}", params.type_name());
    let type_name = cattrs.name().deserialize_name();
    let deny_unknown_fields = cattrs.deny_unknown_fields();

    /// If unknown fields are allowed, we pick the visitor that can
    /// step over those. Otherwise we pick the visitor that fails on
    /// unknown keys.
    let field_visitor_ty = if deny_unknown_fields {
        quote! { _serde::private::de::TagOrContentFieldVisitor }
    } else {
        quote! { _serde::private::de::TagContentOtherFieldVisitor }
    };

    let tag_or_content = quote! {
        #field_visitor_ty {
            tag: #tag,
            content: #content,
        }
    };

    fn is_unit(variant: &Variant) -> bool {
        match variant.style {
            Style::Unit => true,
            Style::Struct | Style::Tuple | Style::Newtype => false,
        }
    }

    let mut missing_content = quote! {
        _serde::export::Err(<__A::Error as _serde::de::Error>::missing_field(#content))
    };
    if variants.iter().any(is_unit) {
        let fallthrough = if variants.iter().all(is_unit) {
            None
        } else {
            Some(
                quote! {
                    _ => #missing_content
                },
            )
        };
        let arms = variants
            .iter()
            .enumerate()
            .filter(|&(_, variant)| !variant.attrs.skip_deserializing() && is_unit(variant),)
            .map(
                |(i, variant)| {
                    let variant_index = field_i(i);
                    let variant_ident = &variant.ident;
                    quote! {
                        __Field::#variant_index => _serde::export::Ok(#this::#variant_ident),
                    }
                },
            );
        missing_content = quote! {
            match __field {
                #(#arms)*
                #fallthrough
            }
        };
    }

    /// Advance the map by one key, returning early in case of error.
    let next_key = quote! {
        try!(_serde::de::MapAccess::next_key_seed(&mut __map, #tag_or_content))
    };

    /// When allowing unknown fields, we want to transparently step through keys we don't care
    /// about until we find `tag`, `content`, or run out of keys.
    let next_relevant_key = if deny_unknown_fields {
        next_key
    } else {
        quote! {
            {
                let mut __rk : _serde::export::Option<_serde::private::de::TagOrContentField> = _serde::export::None;
                while let _serde::export::Some(__k) = #next_key {
                    match __k {
                        _serde::private::de::TagContentOtherField::Other => {
                            try!(_serde::de::MapAccess::next_value::<_serde::de::IgnoredAny>(&mut __map));
                            continue;
                        },
                        _serde::private::de::TagContentOtherField::Tag => {
                            __rk = _serde::export::Some(_serde::private::de::TagOrContentField::Tag);
                            break;
                        }
                        _serde::private::de::TagContentOtherField::Content => {
                            __rk = _serde::export::Some(_serde::private::de::TagOrContentField::Content);
                            break;
                        }
                    }
                }

                __rk
            }
        }
    };

    /// Step through remaining keys, looking for duplicates of previously-seen keys.
    /// When unknown fields are denied, any key that isn't a duplicate will at this
    /// point immediately produce an error.
    let visit_remaining_keys = quote! {
        match #next_relevant_key {
            _serde::export::Some(_serde::private::de::TagOrContentField::Tag) => {
                _serde::export::Err(<__A::Error as _serde::de::Error>::duplicate_field(#tag))
            }
            _serde::export::Some(_serde::private::de::TagOrContentField::Content) => {
                _serde::export::Err(<__A::Error as _serde::de::Error>::duplicate_field(#content))
            }
            _serde::export::None => _serde::export::Ok(__ret),
        }
    };

    quote_block! {
        #variant_visitor

        #variants_stmt

        struct __Seed #de_impl_generics #where_clause {
            field: __Field,
            marker: _serde::export::PhantomData<#this #ty_generics>,
            lifetime: _serde::export::PhantomData<&'de ()>,
        }

        impl #de_impl_generics _serde::de::DeserializeSeed<'de> for __Seed #de_ty_generics #where_clause {
            type Value = #this #ty_generics;

            fn deserialize<__D>(self, __deserializer: __D) -> _serde::export::Result<Self::Value, __D::Error>
                where __D: _serde::Deserializer<'de>
            {
                match self.field {
                    #(#variant_arms)*
                }
            }
        }

        struct __Visitor #de_impl_generics #where_clause {
            marker: _serde::export::PhantomData<#this #ty_generics>,
            lifetime: _serde::export::PhantomData<&'de ()>,
        }

        impl #de_impl_generics _serde::de::Visitor<'de> for __Visitor #de_ty_generics #where_clause {
            type Value = #this #ty_generics;

            fn expecting(&self, formatter: &mut _serde::export::Formatter) -> _serde::export::fmt::Result {
                _serde::export::Formatter::write_str(formatter, #expecting)
            }

            fn visit_map<__A>(self, mut __map: __A) -> _serde::export::Result<Self::Value, __A::Error>
                where __A: _serde::de::MapAccess<'de>
            {
                // Visit the first relevant key.
                match #next_relevant_key {
                    // First key is the tag.
                    _serde::export::Some(_serde::private::de::TagOrContentField::Tag) => {
                        // Parse the tag.
                        let __field = try!(_serde::de::MapAccess::next_value(&mut __map));
                        // Visit the second key.
                        match #next_relevant_key {
                            // Second key is a duplicate of the tag.
                            _serde::export::Some(_serde::private::de::TagOrContentField::Tag) => {
                                _serde::export::Err(<__A::Error as _serde::de::Error>::duplicate_field(#tag))
                            }
                            // Second key is the content.
                            _serde::export::Some(_serde::private::de::TagOrContentField::Content) => {
                                let __ret = try!(_serde::de::MapAccess::next_value_seed(&mut __map,
                                    __Seed {
                                        field: __field,
                                        marker: _serde::export::PhantomData,
                                        lifetime: _serde::export::PhantomData,
                                    }));
                                // Visit remaining keys, looking for duplicates.
                                #visit_remaining_keys
                            }
                            // There is no second key; might be okay if the we have a unit variant.
                            _serde::export::None => #missing_content
                        }
                    }
                    // First key is the content.
                    _serde::export::Some(_serde::private::de::TagOrContentField::Content) => {
                        // Buffer up the content.
                        let __content = try!(_serde::de::MapAccess::next_value::<_serde::private::de::Content>(&mut __map));
                        // Visit the second key.
                        match #next_relevant_key {
                            // Second key is the tag.
                            _serde::export::Some(_serde::private::de::TagOrContentField::Tag) => {
                                let __deserializer = _serde::private::de::ContentDeserializer::<__A::Error>::new(__content);
                                // Parse the tag.
                                let __ret = try!(match try!(_serde::de::MapAccess::next_value(&mut __map)) {
                                    // Deserialize the buffered content now that we know the variant.
                                    #(#variant_arms)*
                                });
                                // Visit remaining keys, looking for duplicates.
                                #visit_remaining_keys
                            }
                            // Second key is a duplicate of the content.
                            _serde::export::Some(_serde::private::de::TagOrContentField::Content) => {
                                _serde::export::Err(<__A::Error as _serde::de::Error>::duplicate_field(#content))
                            }
                            // There is no second key.
                            _serde::export::None => {
                                _serde::export::Err(<__A::Error as _serde::de::Error>::missing_field(#tag))
                            }
                        }
                    }
                    // There is no first key.
                    _serde::export::None => {
                        _serde::export::Err(<__A::Error as _serde::de::Error>::missing_field(#tag))
                    }
                }
            }

            fn visit_seq<__A>(self, mut __seq: __A) -> _serde::export::Result<Self::Value, __A::Error>
                where __A: _serde::de::SeqAccess<'de>
            {
                // Visit the first element - the tag.
                match try!(_serde::de::SeqAccess::next_element(&mut __seq)) {
                    _serde::export::Some(__field) => {
                        // Visit the second element - the content.
                        match try!(_serde::de::SeqAccess::next_element_seed(&mut __seq,
                                __Seed {
                                    field: __field,
                                    marker: _serde::export::PhantomData,
                                    lifetime: _serde::export::PhantomData,
                                })) {
                            _serde::export::Some(__ret) => _serde::export::Ok(__ret),
                            // There is no second element.
                            _serde::export::None => {
                                _serde::export::Err(_serde::de::Error::invalid_length(1, &self))
                            }
                        }
                    }
                    // There is no first element.
                    _serde::export::None => {
                        _serde::export::Err(_serde::de::Error::invalid_length(0, &self))
                    }
                }
            }
        }

        const FIELDS: &'static [&'static str] = &[#tag, #content];
        _serde::Deserializer::deserialize_struct(__deserializer, #type_name, FIELDS,
            __Visitor {
                marker: _serde::export::PhantomData::<#this #ty_generics>,
                lifetime: _serde::export::PhantomData,
            })
    }
}

fn deserialize_untagged_enum(
    params: &Parameters,
    variants: &[Variant],
    cattrs: &attr::Container,
) -> Fragment {
    let attempts = variants
        .iter()
        .filter(|variant| !variant.attrs.skip_deserializing())
        .map(
            |variant| {
                Expr(deserialize_untagged_variant(
                params,
                variant,
                cattrs,
                quote!(_serde::private::de::ContentRefDeserializer::<__D::Error>::new(&__content)),
            ))
            },
        );

    // TODO this message could be better by saving the errors from the failed
    // attempts. The heuristic used by TOML was to count the number of fields
    // processed before an error, and use the error that happened after the
    // largest number of fields. I'm not sure I like that. Maybe it would be
    // better to save all the errors and combine them into one message that
    // explains why none of the variants matched.
    let fallthrough_msg =
        format!("data did not match any variant of untagged enum {}", params.type_name());

    quote_block! {
        let __content = try!(<_serde::private::de::Content as _serde::Deserialize>::deserialize(__deserializer));

        #(
            if let _serde::export::Ok(__ok) = #attempts {
                return _serde::export::Ok(__ok);
            }
        )*

        _serde::export::Err(_serde::de::Error::custom(#fallthrough_msg))
    }
}

fn deserialize_externally_tagged_variant(
    params: &Parameters,
    variant: &Variant,
    cattrs: &attr::Container,
) -> Fragment {
    let variant_ident = &variant.ident;

    match variant.style {
        Style::Unit => {
            let this = &params.this;
            quote_block! {
                try!(_serde::de::VariantAccess::unit_variant(__variant));
                _serde::export::Ok(#this::#variant_ident)
            }
        }
        Style::Newtype => {
            deserialize_externally_tagged_newtype_variant(variant_ident, params, &variant.fields[0])
        }
        Style::Tuple => {
            deserialize_tuple(Some(variant_ident), params, &variant.fields, cattrs, None)
        }
        Style::Struct => {
            deserialize_struct(Some(variant_ident), params, &variant.fields, cattrs, None)
        }
    }
}

fn deserialize_internally_tagged_variant(
    params: &Parameters,
    variant: &Variant,
    cattrs: &attr::Container,
    deserializer: Tokens,
) -> Fragment {
    let variant_ident = &variant.ident;

    match variant.style {
        Style::Unit => {
            let this = &params.this;
            let type_name = params.type_name();
            let variant_name = variant.ident.as_ref();
            quote_block! {
                try!(_serde::Deserializer::deserialize_any(#deserializer, _serde::private::de::InternallyTaggedUnitVisitor::new(#type_name, #variant_name)));
                _serde::export::Ok(#this::#variant_ident)
            }
        }
        Style::Newtype | Style::Struct => {
            deserialize_untagged_variant(params, variant, cattrs, deserializer)
        }
        Style::Tuple => unreachable!("checked in serde_derive_internals"),
    }
}

fn deserialize_untagged_variant(
    params: &Parameters,
    variant: &Variant,
    cattrs: &attr::Container,
    deserializer: Tokens,
) -> Fragment {
    let variant_ident = &variant.ident;

    match variant.style {
        Style::Unit => {
            let this = &params.this;
            let type_name = params.type_name();
            let variant_name = variant.ident.as_ref();
            quote_expr! {
                _serde::export::Result::map(
                    _serde::Deserializer::deserialize_any(
                        #deserializer,
                        _serde::private::de::UntaggedUnitVisitor::new(#type_name, #variant_name)
                    ),
                    |()| #this::#variant_ident)
            }
        }
        Style::Newtype => {
            deserialize_untagged_newtype_variant(
                variant_ident,
                params,
                &variant.fields[0],
                deserializer,
            )
        }
        Style::Tuple => {
            deserialize_tuple(
                Some(variant_ident),
                params,
                &variant.fields,
                cattrs,
                Some(deserializer),
            )
        }
        Style::Struct => {
            deserialize_struct(
                Some(variant_ident),
                params,
                &variant.fields,
                cattrs,
                Some(deserializer),
            )
        }
    }
}

fn deserialize_externally_tagged_newtype_variant(
    variant_ident: &syn::Ident,
    params: &Parameters,
    field: &Field,
) -> Fragment {
    let this = &params.this;
    match field.attrs.deserialize_with() {
        None => {
            let field_ty = &field.ty;
            quote_expr! {
                _serde::export::Result::map(
                    _serde::de::VariantAccess::newtype_variant::<#field_ty>(__variant),
                    #this::#variant_ident)
            }
        }
        Some(path) => {
            let (wrapper, wrapper_ty) = wrap_deserialize_with(params, field.ty, path);
            quote_block! {
                #wrapper
                _serde::export::Result::map(
                    _serde::de::VariantAccess::newtype_variant::<#wrapper_ty>(__variant),
                    |__wrapper| #this::#variant_ident(__wrapper.value))
            }
        }
    }
}

fn deserialize_untagged_newtype_variant(
    variant_ident: &syn::Ident,
    params: &Parameters,
    field: &Field,
    deserializer: Tokens,
) -> Fragment {
    let this = &params.this;
    match field.attrs.deserialize_with() {
        None => {
            let field_ty = &field.ty;
            quote_expr! {
                _serde::export::Result::map(
                    <#field_ty as _serde::Deserialize>::deserialize(#deserializer),
                    #this::#variant_ident)
            }
        }
        Some(path) => {
            let (wrapper, wrapper_ty) = wrap_deserialize_with(params, field.ty, path);
            quote_block! {
                #wrapper
                _serde::export::Result::map(
                    <#wrapper_ty as _serde::Deserialize>::deserialize(#deserializer),
                    |__wrapper| #this::#variant_ident(__wrapper.value))
            }
        }
    }
}

fn deserialize_generated_identifier(
    fields: Vec<(String, Ident)>,
    cattrs: &attr::Container,
    is_variant: bool,
) -> Fragment {
    let this = quote!(__Field);
    let field_idents: &Vec<_> = &fields.iter().map(|&(_, ref ident)| ident).collect();

    let (ignore_variant, fallthrough) = if is_variant || cattrs.deny_unknown_fields() {
        (None, None)
    } else {
        let ignore_variant = quote!(__ignore,);
        let fallthrough = quote!(_serde::export::Ok(__Field::__ignore));
        (Some(ignore_variant), Some(fallthrough))
    };

    let visitor_impl = Stmts(deserialize_identifier(this, &fields, is_variant, fallthrough),);

    quote_block! {
        #[allow(non_camel_case_types)]
        enum __Field {
            #(#field_idents,)*
            #ignore_variant
        }

        struct __FieldVisitor;

        impl<'de> _serde::de::Visitor<'de> for __FieldVisitor {
            type Value = __Field;

            #visitor_impl
        }

        impl<'de> _serde::Deserialize<'de> for __Field {
            #[inline]
            fn deserialize<__D>(__deserializer: __D) -> _serde::export::Result<Self, __D::Error>
                where __D: _serde::Deserializer<'de>
            {
                _serde::Deserializer::deserialize_identifier(__deserializer, __FieldVisitor)
            }
        }
    }
}

fn deserialize_custom_identifier(
    params: &Parameters,
    variants: &[Variant],
    cattrs: &attr::Container,
) -> Fragment {
    let is_variant = match cattrs.identifier() {
        attr::Identifier::Variant => true,
        attr::Identifier::Field => false,
        attr::Identifier::No => unreachable!(),
    };

    let this = &params.this;
    let this = quote!(#this);

    let (ordinary, fallthrough) = if let Some(last) = variants.last() {
        let last_ident = &last.ident;
        if last.attrs.other() {
            let ordinary = &variants[..variants.len() - 1];
            let fallthrough = quote!(_serde::export::Ok(#this::#last_ident));
            (ordinary, Some(fallthrough))
        } else if let Style::Newtype = last.style {
            let ordinary = &variants[..variants.len() - 1];
            let deserializer = quote!(_serde::private::de::IdentifierDeserializer::from(__value));
            let fallthrough = quote! {
                _serde::export::Result::map(
                    _serde::Deserialize::deserialize(#deserializer),
                    #this::#last_ident)
            };
            (ordinary, Some(fallthrough))
        } else {
            (variants, None)
        }
    } else {
        (variants, None)
    };

    let names_idents: Vec<_> = ordinary
        .iter()
        .map(|variant| (variant.attrs.name().deserialize_name(), variant.ident.clone()),)
        .collect();

    let names = names_idents.iter().map(|&(ref name, _)| name);

    let names_const = if fallthrough.is_some() {
        None
    } else if is_variant {
        let variants = quote! {
            const VARIANTS: &'static [&'static str] = &[ #(#names),* ];
        };
        Some(variants)
    } else {
        let fields = quote! {
            const FIELDS: &'static [&'static str] = &[ #(#names),* ];
        };
        Some(fields)
    };

    let (de_impl_generics, de_ty_generics, ty_generics, where_clause) = split_with_de_lifetime(params,);
    let visitor_impl =
        Stmts(deserialize_identifier(this.clone(), &names_idents, is_variant, fallthrough),);

    quote_block! {
        #names_const

        struct __FieldVisitor #de_impl_generics #where_clause {
            marker: _serde::export::PhantomData<#this #ty_generics>,
            lifetime: _serde::export::PhantomData<&'de ()>,
        }

        impl #de_impl_generics _serde::de::Visitor<'de> for __FieldVisitor #de_ty_generics #where_clause {
            type Value = #this #ty_generics;

            #visitor_impl
        }

        let __visitor = __FieldVisitor {
            marker: _serde::export::PhantomData::<#this #ty_generics>,
            lifetime: _serde::export::PhantomData,
        };
        _serde::Deserializer::deserialize_identifier(__deserializer, __visitor)
    }
}

fn deserialize_identifier(
    this: Tokens,
    fields: &[(String, Ident)],
    is_variant: bool,
    fallthrough: Option<Tokens>,
) -> Fragment {
    let field_strs = fields.iter().map(|&(ref name, _)| name);
    let field_bytes = fields.iter().map(|&(ref name, _)| quote::ByteStr(name));

    let constructors: &Vec<_> = &fields
                                     .iter()
                                     .map(|&(_, ref ident)| quote!(#this::#ident))
                                     .collect();

    let expecting = if is_variant {
        "variant identifier"
    } else {
        "field identifier"
    };

    let visit_index = if is_variant {
        let variant_indices = 0u32..;
        let fallthrough_msg = format!("variant index 0 <= i < {}", fields.len());
        let visit_index = quote! {
            fn visit_u32<__E>(self, __value: u32) -> _serde::export::Result<Self::Value, __E>
                where __E: _serde::de::Error
            {
                match __value {
                    #(
                        #variant_indices => _serde::export::Ok(#constructors),
                    )*
                    _ => _serde::export::Err(_serde::de::Error::invalid_value(
                                _serde::de::Unexpected::Unsigned(__value as u64),
                                &#fallthrough_msg))
                }
            }
        };
        Some(visit_index)
    } else {
        None
    };

    let bytes_to_str = if fallthrough.is_some() {
        None
    } else {
        let conversion = quote! {
            let __value = &_serde::export::from_utf8_lossy(__value);
        };
        Some(conversion)
    };

    let fallthrough_arm = if let Some(fallthrough) = fallthrough {
        fallthrough
    } else if is_variant {
        quote! {
            _serde::export::Err(_serde::de::Error::unknown_variant(__value, VARIANTS))
        }
    } else {
        quote! {
            _serde::export::Err(_serde::de::Error::unknown_field(__value, FIELDS))
        }
    };

    quote_block! {
        fn expecting(&self, formatter: &mut _serde::export::Formatter) -> _serde::export::fmt::Result {
            _serde::export::Formatter::write_str(formatter, #expecting)
        }

        #visit_index

        fn visit_str<__E>(self, __value: &str) -> _serde::export::Result<Self::Value, __E>
            where __E: _serde::de::Error
        {
            match __value {
                #(
                    #field_strs => _serde::export::Ok(#constructors),
                )*
                _ => #fallthrough_arm
            }
        }

        fn visit_bytes<__E>(self, __value: &[u8]) -> _serde::export::Result<Self::Value, __E>
            where __E: _serde::de::Error
        {
            match __value {
                #(
                    #field_bytes => _serde::export::Ok(#constructors),
                )*
                _ => {
                    #bytes_to_str
                    #fallthrough_arm
                }
            }
        }
    }
}

fn deserialize_struct_visitor(
    struct_path: Tokens,
    params: &Parameters,
    fields: &[Field],
    cattrs: &attr::Container,
) -> (Fragment, Fragment, Fragment) {
    let field_names_idents: Vec<_> = fields
        .iter()
        .enumerate()
        .filter(|&(_, field)| !field.attrs.skip_deserializing())
        .map(|(i, field)| (field.attrs.name().deserialize_name(), field_i(i)),)
        .collect();

    let fields_stmt = {
        let field_names = field_names_idents.iter().map(|&(ref name, _)| name);
        quote_block! {
            const FIELDS: &'static [&'static str] = &[ #(#field_names),* ];
        }
    };

    let field_visitor = deserialize_generated_identifier(field_names_idents, cattrs, false);

    let visit_map = deserialize_map(struct_path, params, fields, cattrs);

    (field_visitor, fields_stmt, visit_map)
}

fn deserialize_map(
    struct_path: Tokens,
    params: &Parameters,
    fields: &[Field],
    cattrs: &attr::Container,
) -> Fragment {
    // Create the field names for the fields.
    let fields_names: Vec<_> = fields
        .iter()
        .enumerate()
        .map(|(i, field)| (field, field_i(i)))
        .collect();

    // Declare each field that will be deserialized.
    let let_values = fields_names
        .iter()
        .filter(|&&(field, _)| !field.attrs.skip_deserializing())
        .map(
            |&(field, ref name)| {
                let field_ty = &field.ty;
                quote! {
                    let mut #name: _serde::export::Option<#field_ty> = _serde::export::None;
                }
            },
        );

    // Match arms to extract a value for a field.
    let value_arms = fields_names.iter()
        .filter(|&&(field, _)| !field.attrs.skip_deserializing())
        .map(|&(field, ref name)| {
            let deser_name = field.attrs.name().deserialize_name();

            let visit = match field.attrs.deserialize_with() {
                None => {
                    let field_ty = &field.ty;
                    quote! {
                        try!(_serde::de::MapAccess::next_value::<#field_ty>(&mut __map))
                    }
                }
                Some(path) => {
                    let (wrapper, wrapper_ty) = wrap_deserialize_with(
                        params, field.ty, path);
                    quote!({
                        #wrapper
                        try!(_serde::de::MapAccess::next_value::<#wrapper_ty>(&mut __map)).value
                    })
                }
            };
            quote! {
                __Field::#name => {
                    if _serde::export::Option::is_some(&#name) {
                        return _serde::export::Err(<__A::Error as _serde::de::Error>::duplicate_field(#deser_name));
                    }
                    #name = _serde::export::Some(#visit);
                }
            }
        });

    // Visit ignored values to consume them
    let ignored_arm = if cattrs.deny_unknown_fields() {
        None
    } else {
        Some(quote! {
            _ => { let _ = try!(_serde::de::MapAccess::next_value::<_serde::de::IgnoredAny>(&mut __map)); }
        })
    };

    let all_skipped = fields
        .iter()
        .all(|field| field.attrs.skip_deserializing());
    let match_keys = if cattrs.deny_unknown_fields() && all_skipped {
        quote! {
            // FIXME: Once we drop support for Rust 1.15:
            // let _serde::export::None::<__Field> = try!(_serde::de::MapAccess::next_key(&mut __map));
            _serde::export::Option::map(
                try!(_serde::de::MapAccess::next_key::<__Field>(&mut __map)),
                |__impossible| match __impossible {});
        }
    } else {
        quote! {
            while let _serde::export::Some(__key) = try!(_serde::de::MapAccess::next_key::<__Field>(&mut __map)) {
                match __key {
                    #(#value_arms)*
                    #ignored_arm
                }
            }
        }
    };

    let extract_values = fields_names
        .iter()
        .filter(|&&(field, _)| !field.attrs.skip_deserializing())
        .map(
            |&(field, ref name)| {
                let missing_expr = Match(expr_is_missing(&field, cattrs));

                quote! {
                    let #name = match #name {
                        _serde::export::Some(#name) => #name,
                        _serde::export::None => #missing_expr
                    };
                }
            },
        );

    let result = fields_names
        .iter()
        .map(
            |&(field, ref name)| {
                let ident = field
                    .ident
                    .clone()
                    .expect("struct contains unnamed fields");
                if field.attrs.skip_deserializing() {
                    let value = Expr(expr_is_missing(&field, cattrs));
                    quote!(#ident: #value)
                } else {
                    quote!(#ident: #name)
                }
            },
        );

    let let_default = match *cattrs.default() {
        attr::Default::Default => {
            Some(
                quote!(
                let __default: Self::Value = _serde::export::Default::default();
            ),
            )
        }
        attr::Default::Path(ref path) => {
            Some(
                quote!(
                let __default: Self::Value = #path();
            ),
            )
        }
        attr::Default::None => {
            // We don't need the default value, to prevent an unused variable warning
            // we'll leave the line empty.
            None
        }
    };

    let mut result = quote!(#struct_path { #(#result),* });
    if params.has_getter {
        let this = &params.this;
        result = quote! {
            _serde::export::Into::<#this>::into(#result)
        };
    }

    quote_block! {
        #(#let_values)*

        #match_keys

        #let_default

        #(#extract_values)*

        _serde::export::Ok(#result)
    }
}

fn field_i(i: usize) -> Ident {
    Ident::new(format!("__field{}", i))
}

/// This function wraps the expression in `#[serde(deserialize_with = "...")]`
/// in a trait to prevent it from accessing the internal `Deserialize` state.
fn wrap_deserialize_with(
    params: &Parameters,
    field_ty: &syn::Ty,
    deserialize_with: &syn::Path,
) -> (Tokens, Tokens) {
    let this = &params.this;
    let (de_impl_generics, de_ty_generics, ty_generics, where_clause) = split_with_de_lifetime(params,);

    let wrapper = quote! {
        struct __DeserializeWith #de_impl_generics #where_clause {
            value: #field_ty,
            phantom: _serde::export::PhantomData<#this #ty_generics>,
            lifetime: _serde::export::PhantomData<&'de ()>,
        }

        impl #de_impl_generics _serde::Deserialize<'de> for __DeserializeWith #de_ty_generics #where_clause {
            fn deserialize<__D>(__deserializer: __D) -> _serde::export::Result<Self, __D::Error>
                where __D: _serde::Deserializer<'de>
            {
                _serde::export::Ok(__DeserializeWith {
                    value: try!(#deserialize_with(__deserializer)),
                    phantom: _serde::export::PhantomData,
                    lifetime: _serde::export::PhantomData,
                })
            }
        }
    };

    let wrapper_ty = quote!(__DeserializeWith #de_ty_generics);

    (wrapper, wrapper_ty)
}

fn expr_is_missing(field: &Field, cattrs: &attr::Container) -> Fragment {
    match *field.attrs.default() {
        attr::Default::Default => {
            return quote_expr!(_serde::export::Default::default());
        }
        attr::Default::Path(ref path) => {
            return quote_expr!(#path());
        }
        attr::Default::None => { /* below */ }
    }

    match *cattrs.default() {
        attr::Default::Default |
        attr::Default::Path(_) => {
            let ident = &field.ident;
            return quote_expr!(__default.#ident);
        }
        attr::Default::None => { /* below */ }
    }

    let name = field.attrs.name().deserialize_name();
    match field.attrs.deserialize_with() {
        None => {
            quote_expr! {
                try!(_serde::private::de::missing_field(#name))
            }
        }
        Some(_) => {
            quote_expr! {
                return _serde::export::Err(<__A::Error as _serde::de::Error>::missing_field(#name))
            }
        }
    }
}

struct DeImplGenerics<'a>(&'a Parameters);

impl<'a> ToTokens for DeImplGenerics<'a> {
    fn to_tokens(&self, tokens: &mut Tokens) {
        let mut generics = self.0.generics.clone();
        generics.lifetimes.insert(0, self.0.de_lifetime_def());
        let (impl_generics, _, _) = generics.split_for_impl();
        impl_generics.to_tokens(tokens);
    }
}

struct DeTyGenerics<'a>(&'a syn::Generics);

impl<'a> ToTokens for DeTyGenerics<'a> {
    fn to_tokens(&self, tokens: &mut Tokens) {
        let mut generics = self.0.clone();
        generics
            .lifetimes
            .insert(0, syn::LifetimeDef::new("'de"));
        let (_, ty_generics, _) = generics.split_for_impl();
        ty_generics.to_tokens(tokens);
    }
}

fn split_with_de_lifetime(params: &Parameters,)
    -> (DeImplGenerics, DeTyGenerics, syn::TyGenerics, &syn::WhereClause) {
    let de_impl_generics = DeImplGenerics(&params);
    let de_ty_generics = DeTyGenerics(&params.generics);
    let (_, ty_generics, where_clause) = params.generics.split_for_impl();
    (de_impl_generics, de_ty_generics, ty_generics, where_clause)
}
