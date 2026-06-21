//! Deserialization for externally tagged enums:
//!
//! ```ignore
//! enum Enum {}
//! ```

use crate::de::struct_;
use crate::de::tuple;
use crate::de::{
    expr_is_missing, unwrap_to_variant_closure, wrap_deserialize_field_with, wrap_deserialize_with,
    Parameters, StructForm, TupleForm,
};
use crate::fragment::{Expr, Fragment, Match};
use crate::internals::ast::{Field, Style, Variant};
use crate::internals::attr;
use crate::private;
use proc_macro2::TokenStream;
use quote::{quote, quote_spanned};
use syn::spanned::Spanned;

/// Generates `Deserialize::deserialize` body for an `enum Enum {...}` without additional attributes
pub(super) fn deserialize(
    params: &Parameters,
    variants: &[Variant],
    cattrs: &attr::Container,
) -> Fragment {
    let this_type = &params.this_type;
    let (de_impl_generics, de_ty_generics, ty_generics, where_clause) =
        params.generics_with_de_lifetime();
    let delife = params.borrowed.de_lifetime();

    let type_name = cattrs.name().deserialize_name();
    let expecting = format!("enum {}", params.type_name());
    let expecting = cattrs.expecting().unwrap_or(&expecting);

    // Match arms to extract a variant from a string
    let variant_arms = variants
        .iter()
        .filter(|variant| !variant.attrs.skip_deserializing())
        .enumerate()
        .map(|(i, variant)| {
            let block = Match(deserialize_externally_tagged_variant(
                params, variant, cattrs,
            ));

            quote! {
                _serde::#private::Ok((#i, __variant)) => #block
            }
        });

    let seed = match variants
        .iter()
        .filter(|variant| !variant.attrs.skip_deserializing())
        .position(|variant| variant.attrs.other())
    {
        Some(other) => {
            quote!(_serde::#private::de::VariantOtherSeed::new(VARIANT_ALIASES, #other))
        }
        None => quote!(_serde::#private::de::VariantSeed::new(
            VARIANT_ALIASES,
            VARIANTS
        )),
    };

    let variant_names = variants
        .iter()
        .filter(|variant| !variant.attrs.skip_deserializing())
        .flat_map(|variant| variant.attrs.aliases());
    let aliases = variants.iter().filter_map(|variant| {
        if variant.attrs.skip_deserializing() {
            None
        } else {
            let aliases = variant.attrs.aliases();
            Some(quote!(&[ #(#aliases),* ]))
        }
    });

    quote_block! {
        #[doc(hidden)]
        struct __Visitor #de_impl_generics #where_clause {
            marker: _serde::#private::PhantomData<#this_type #ty_generics>,
            lifetime: _serde::#private::PhantomData<&#delife ()>,
        }

        #[automatically_derived]
        impl #de_impl_generics _serde::de::Visitor<#delife> for __Visitor #de_ty_generics #where_clause {
            type Value = #this_type #ty_generics;

            fn expecting(&self, __formatter: &mut _serde::#private::Formatter) -> _serde::#private::fmt::Result {
                _serde::#private::Formatter::write_str(__formatter, #expecting)
            }

            fn visit_enum<__A>(self, __data: __A) -> _serde::#private::Result<Self::Value, __A::Error>
            where
                __A: _serde::de::EnumAccess<#delife>,
            {
                match _serde::de::EnumAccess::variant_seed(__data, #seed) {
                    #(#variant_arms)*
                    _serde::#private::Err(__err) => _serde::#private::Err(__err),
                    _ => unreachable!(),
                }
            }
        }

        #[doc(hidden)]
        const VARIANTS: &'static [&'static str] = &[ #(#variant_names),* ];
        #[doc(hidden)]
        const VARIANT_ALIASES: &[&[&str]] = &[ #(#aliases),* ];

        _serde::Deserializer::deserialize_enum(
            __deserializer,
            #type_name,
            VARIANTS,
            __Visitor {
                marker: _serde::#private::PhantomData::<#this_type #ty_generics>,
                lifetime: _serde::#private::PhantomData,
            },
        )
    }
}

fn deserialize_externally_tagged_variant(
    params: &Parameters,
    variant: &Variant,
    cattrs: &attr::Container,
) -> Fragment {
    if let Some(path) = variant.attrs.deserialize_with() {
        let (wrapper, wrapper_ty, unwrap_fn) = wrap_deserialize_variant_with(params, variant, path);
        return quote_block! {
            #wrapper
            _serde::#private::Result::map(
                _serde::de::VariantAccess::newtype_variant::<#wrapper_ty>(__variant), #unwrap_fn)
        };
    }

    let variant_ident = &variant.ident;

    match variant.style {
        Style::Unit => {
            let this_value = &params.this_value;
            quote_block! {
                _serde::de::VariantAccess::unit_variant(__variant)?;
                _serde::#private::Ok(#this_value::#variant_ident)
            }
        }
        Style::Newtype => deserialize_externally_tagged_newtype_variant(
            variant_ident,
            params,
            &variant.fields[0],
            cattrs,
        ),
        Style::Tuple => tuple::deserialize(
            params,
            &variant.fields,
            cattrs,
            TupleForm::ExternallyTagged(variant_ident),
        ),
        Style::Struct => struct_::deserialize(
            params,
            &variant.fields,
            cattrs,
            StructForm::ExternallyTagged(variant_ident),
        ),
    }
}

fn wrap_deserialize_variant_with(
    params: &Parameters,
    variant: &Variant,
    deserialize_with: &syn::ExprPath,
) -> (TokenStream, TokenStream, TokenStream) {
    let field_tys = variant.fields.iter().map(|field| field.ty);
    let (wrapper, wrapper_ty) =
        wrap_deserialize_with(params, &quote!((#(#field_tys),*)), deserialize_with);

    let unwrap_fn = unwrap_to_variant_closure(params, variant, true);

    (wrapper, wrapper_ty, unwrap_fn)
}

fn deserialize_externally_tagged_newtype_variant(
    variant_ident: &syn::Ident,
    params: &Parameters,
    field: &Field,
    cattrs: &attr::Container,
) -> Fragment {
    let this_value = &params.this_value;

    if field.attrs.skip_deserializing() {
        let default = Expr(expr_is_missing(field, cattrs));
        return quote_block! {
            _serde::de::VariantAccess::unit_variant(__variant)?;
            _serde::#private::Ok(#this_value::#variant_ident(#default))
        };
    }

    match field.attrs.deserialize_with() {
        None => {
            let field_ty = field.ty;
            let span = field.original.span();
            let func =
                quote_spanned!(span=> _serde::de::VariantAccess::newtype_variant::<#field_ty>);
            quote_expr! {
                _serde::#private::Result::map(#func(__variant), #this_value::#variant_ident)
            }
        }
        Some(path) => {
            let (wrapper, wrapper_ty) = wrap_deserialize_field_with(params, field.ty, path);
            quote_block! {
                #wrapper
                _serde::#private::Result::map(
                    _serde::de::VariantAccess::newtype_variant::<#wrapper_ty>(__variant),
                    |__wrapper| #this_value::#variant_ident(__wrapper.value))
            }
        }
    }
}
