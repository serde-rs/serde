//! Deserialization for internally tagged enums:
//!
//! ```ignore
//! #[serde(tag = "...")]
//! enum Enum {}
//! ```

use crate::de::enum_;
use crate::de::enum_untagged;
use crate::de::struct_;
use crate::de::{
    effective_style, expr_is_missing, field_i, unwrap_to_variant_closure, Parameters, StructForm,
};
use crate::fragment::{Expr, Fragment, Match};
use crate::internals::ast::{Style, Variant};
use crate::internals::attr;
use crate::private;
use quote::quote;

/// Generates `Deserialize::deserialize` body for an `enum Enum {...}` with `#[serde(tag)]` attribute
pub(super) fn deserialize(
    params: &Parameters,
    variants: &[Variant],
    cattrs: &attr::Container,
    tag: &str,
) -> Fragment {
    let (variants_stmt, variant_visitor) = enum_::prepare_enum_variant_enum(variants);

    // Match arms to extract a variant from a string
    let variant_arms = variants
        .iter()
        .enumerate()
        .filter(|&(_, variant)| !variant.attrs.skip_deserializing())
        .map(|(i, variant)| {
            let variant_name = field_i(i);

            let block = Match(deserialize_internally_tagged_variant(
                params, variant, cattrs,
            ));

            quote! {
                __Field::#variant_name => #block
            }
        });

    let expecting = format!("internally tagged enum {}", params.type_name());
    let expecting = cattrs.expecting().unwrap_or(&expecting);

    let this_type = &params.this_type;
    let (de_impl_generics, de_ty_generics, ty_generics, where_clause) =
        params.generics_with_de_lifetime();
    let delife = params.borrowed.de_lifetime();

    quote_block! {
        #variant_visitor

        #variants_stmt

        struct __Seed #de_impl_generics #where_clause {
            tag: __Field,
            marker: _serde::#private::PhantomData<#this_type #ty_generics>,
            lifetime: _serde::#private::PhantomData<&#delife ()>,
        }

        impl #de_impl_generics _serde::de::Deserialize<#delife> for __Seed #de_ty_generics #where_clause {
            fn deserialize<__D>(__deserializer: __D) -> _serde::#private::Result<Self, __D::Error>
            where
                __D: _serde::de::Deserializer<#delife>,
            {
                _serde::#private::Result::map(
                    __Field::deserialize(__deserializer),
                    |__tag| __Seed {
                        tag: __tag,
                        marker: _serde::#private::PhantomData,
                        lifetime: _serde::#private::PhantomData,
                    }
                )
            }
        }

        impl #de_impl_generics _serde::de::DeserializeSeed<#delife> for __Seed #de_ty_generics #where_clause {
            type Value = #this_type #ty_generics;

            fn deserialize<__D>(self, __deserializer: __D) -> _serde::#private::Result<Self::Value, __D::Error>
            where
                __D: _serde::de::Deserializer<#delife>,
            {
                match self.tag {
                    #(#variant_arms)*
                }
            }
        }

        _serde::Deserializer::deserialize_any(
            __deserializer,
            _serde::#private::de::TaggedContentVisitor::<__Seed>::new(#tag, #expecting)
        )
    }
}

// Generates significant part of the visit_seq and visit_map bodies of visitors
// for the variants of internally tagged enum.
fn deserialize_internally_tagged_variant(
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
            let default = variant.fields.first().map(|field| {
                let default = Expr(expr_is_missing(field, cattrs));
                quote!((#default))
            });
            quote_block! {
                _serde::Deserializer::deserialize_any(__deserializer, _serde::#private::de::InternallyTaggedUnitVisitor::new(#type_name, #variant_name))?;
                _serde::#private::Ok(#this_value::#variant_ident #default)
            }
        }
        Style::Newtype => {
            enum_untagged::deserialize_newtype_variant(variant_ident, params, &variant.fields[0])
        }
        Style::Struct => struct_::deserialize(
            params,
            &variant.fields,
            cattrs,
            StructForm::InternallyTagged(variant_ident),
        ),
        Style::Tuple => unreachable!("checked in serde_derive_internals"),
    }
}
