//! Deserialization for untagged enums:
//!
//! ```ignore
//! #[serde(untagged)]
//! enum Enum {}
//! ```

use crate::de::enum_;
use crate::de::struct_;
use crate::de::tuple;
use crate::de::{
    effective_style, expr_is_missing, field_i, unwrap_to_variant_closure, Parameters, StructForm,
    TupleForm,
};
use crate::fragment::{Expr, Fragment};
use crate::internals::ast::{Style, Variant};
use crate::internals::attr;
use crate::private;
use proc_macro2::TokenStream;
use quote::{quote, quote_spanned};
use syn::spanned::Spanned;

/// Generates `Deserialize::deserialize` body for an `enum Enum {...}` with `#[serde(untagged)]` attribute
pub(super) fn deserialize(
    params: &Parameters,
    variants: &[Variant],
    cattrs: &attr::Container,
    first_attempt: Option<TokenStream>,
) -> Fragment {
    let attempts = variants
        .iter()
        .filter(|variant| !variant.attrs.skip_deserializing())
        .map(|variant| Expr(deserialize_variant(params, variant, cattrs)));
    // TODO this message could be better by saving the errors from the failed
    // attempts. The heuristic used by TOML was to count the number of fields
    // processed before an error, and use the error that happened after the
    // largest number of fields. I'm not sure I like that. Maybe it would be
    // better to save all the errors and combine them into one message that
    // explains why none of the variants matched.
    let fallthrough_msg = format!(
        "data did not match any variant of untagged enum {}",
        params.type_name()
    );
    let fallthrough_msg = cattrs.expecting().unwrap_or(&fallthrough_msg);

    let private2 = private;
    quote_block! {
        let __content = _serde::de::DeserializeSeed::deserialize(_serde::#private::de::ContentVisitor::new(), __deserializer)?;
        let __deserializer = _serde::#private::de::ContentRefDeserializer::<__D::Error>::new(&__content);

        #first_attempt

        #(
            if let _serde::#private2::Ok(__ok) = #attempts {
                return _serde::#private2::Ok(__ok);
            }
        )*

        _serde::#private::Err(_serde::de::Error::custom(#fallthrough_msg))
    }
}

// Also used by adjacently tagged enums
pub(super) fn deserialize_variant(
    params: &Parameters,
    variant: &Variant,
    cattrs: &attr::Container,
) -> Fragment {
    if let Some(path) = variant.attrs.deserialize_with() {
        let unwrap_fn = unwrap_to_variant_closure(params, variant, false);
        return quote_block! {
            _serde::#private::Result::map(#path(__deserializer), #unwrap_fn)
        };
    }

    let variant_ident = &variant.ident;

    match effective_style(variant) {
        Style::Unit => {
            let this_value = &params.this_value;
            let type_name = params.type_name();
            let variant_name = variant.ident.to_string();
            let default = enum_::construct_default_tuple(variant, cattrs);
            quote_expr! {
                match _serde::Deserializer::deserialize_any(
                    __deserializer,
                    _serde::#private::de::UntaggedUnitVisitor::new(#type_name, #variant_name)
                ) {
                    _serde::#private::Ok(()) => _serde::#private::Ok(#this_value::#variant_ident #default),
                    _serde::#private::Err(__err) => _serde::#private::Err(__err),
                }
            }
        }
        Style::Newtype => deserialize_newtype_variant(params, variant, cattrs),
        Style::Tuple => tuple::deserialize(
            params,
            &variant.fields,
            cattrs,
            TupleForm::Untagged(variant_ident),
        ),
        Style::Struct => struct_::deserialize(
            params,
            &variant.fields,
            cattrs,
            StructForm::Untagged(variant_ident),
        ),
    }
}

// Also used by internally tagged enums
// Implicitly (via `generate_variant`) used by adjacently tagged enums
pub(super) fn deserialize_newtype_variant(
    params: &Parameters,
    variant: &Variant,
    cattrs: &attr::Container,
) -> Fragment {
    let this_value = &params.this_value;
    let variant_ident = &variant.ident;

    let fields = variant.fields.iter().enumerate().map(|(i, _)| field_i(i));
    let define = variant.fields.iter().enumerate().filter_map(|(i, field)| {
        if field.attrs.skip_deserializing() {
            let name = field_i(i);
            let field_ty = field.ty;
            // This expression always will generate access to default implementation --
            // see comments on `expr_is_missing`
            let expr = Expr(expr_is_missing(field, cattrs));
            return Some(quote!(let #name: #field_ty = #expr;));
        }
        None
    });
    // We deserialize newtype struct, so only one field is not skipped
    let (i, field) = variant
        .fields
        .iter()
        .enumerate()
        .find(|(_, field)| !field.attrs.skip_deserializing())
        .expect("checked in Variant::de_style()");

    let name = field_i(i);
    let expr = match field.attrs.deserialize_with() {
        None => {
            let span = field.original.span();
            quote_spanned!(span=> _serde::Deserialize::deserialize)
        }
        Some(path) => quote!(#path),
    };

    quote_expr! {
        match #expr(__deserializer) {
            _serde::#private::Ok(#name) => {
                #(#define)*
                _serde::#private::Ok(#this_value::#variant_ident(
                    #(#fields,)*
                ))
            }
            _serde::#private::Err(__err) => _serde::#private::Err(__err),
        }
    }
}
