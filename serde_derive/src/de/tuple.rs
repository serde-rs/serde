#[cfg(feature = "deserialize_in_place")]
use crate::de::{expr_is_missing, place_lifetime, read_fields_in_order_in_place};
use crate::de::{has_flatten, read_fields_in_order, read_from_seq_access, Parameters, TupleForm};
#[cfg(feature = "deserialize_in_place")]
use crate::fragment::Expr;
use crate::fragment::{Fragment, Stmts};
use crate::internals::ast::Field;
use crate::internals::attr;
use crate::private;
use quote::{quote, quote_spanned};
use syn::spanned::Spanned;
#[cfg(feature = "deserialize_in_place")]
use syn::Index;

/// Generates `Deserialize::deserialize` body for a `struct Tuple(...);` including `struct Newtype(T);`
pub(super) fn deserialize(
    params: &Parameters,
    fields: &[Field],
    cattrs: &attr::Container,
    form: TupleForm,
) -> Fragment {
    assert!(
        !has_flatten(fields),
        "tuples and tuple variants cannot have flatten fields"
    );

    let field_count = fields
        .iter()
        .filter(|field| !field.attrs.skip_deserializing())
        .count();

    let this_type = &params.this_type;
    let this_value = &params.this_value;
    let (de_impl_generics, de_ty_generics, ty_generics, where_clause) =
        params.generics_with_de_lifetime();
    let delife = params.borrowed.de_lifetime();

    // If there are getters (implying private fields), construct the local type
    // and use an `Into` conversion to get the remote type. If there are no
    // getters then construct the target type directly.
    let construct = if params.has_getter {
        let local = &params.local;
        quote!(#local)
    } else {
        quote!(#this_value)
    };

    let type_path = match form {
        TupleForm::Tuple => construct,
        TupleForm::ExternallyTagged(variant_ident) | TupleForm::Untagged(variant_ident) => {
            quote!(#construct::#variant_ident)
        }
    };
    let expecting = match form {
        TupleForm::Tuple => format!("tuple struct {}", params.type_name()),
        TupleForm::ExternallyTagged(variant_ident) | TupleForm::Untagged(variant_ident) => {
            format!("tuple variant {}::{}", params.type_name(), variant_ident)
        }
    };
    let expecting = cattrs.expecting().unwrap_or(&expecting);

    let nfields = fields.len();

    let visit_newtype_struct = match form {
        TupleForm::Tuple if field_count == 1 => {
            let visit_newtype_struct = Stmts(read_fields_in_order(
                &type_path,
                params,
                fields,
                false,
                cattrs,
                expecting,
                |_, _, field, _, _| {
                    let deserialize = match field.attrs.deserialize_with() {
                        None => {
                            let field_ty = field.ty;

                            let span = field.original.span();
                            quote_spanned!(span=> <#field_ty as _serde::Deserialize>::deserialize)
                        }
                        Some(path) => {
                            // If #path returns wrong type, error will be reported here (^^^^^).
                            // We attach span of the path to the function so it will be reported
                            // on the #[serde(with = "...")]
                            //                       ^^^^^
                            quote_spanned!(path.span()=> #path)
                        }
                    };
                    // __e cannot be in quote_spanned! because of macro hygiene
                    quote!(#deserialize(__e)?)
                },
            ));

            Some(quote! {
                #[inline]
                fn visit_newtype_struct<__E>(self, __e: __E) -> _serde::#private::Result<Self::Value, __E::Error>
                where
                    __E: _serde::Deserializer<#delife>,
                {
                    #visit_newtype_struct
                }
            })
        }
        _ => None,
    };

    let visit_seq = Stmts(read_fields_in_order(
        &type_path,
        params,
        fields,
        false,
        cattrs,
        expecting,
        read_from_seq_access,
    ));

    let visitor_expr = quote! {
        __Visitor {
            marker: _serde::#private::PhantomData::<#this_type #ty_generics>,
            lifetime: _serde::#private::PhantomData,
        }
    };
    let dispatch = match form {
        TupleForm::Tuple if field_count != 0 && nfields == 1 => {
            let type_name = cattrs.name().deserialize_name();
            quote! {
                _serde::Deserializer::deserialize_newtype_struct(__deserializer, #type_name, #visitor_expr)
            }
        }
        TupleForm::Tuple => {
            let type_name = cattrs.name().deserialize_name();
            quote! {
                _serde::Deserializer::deserialize_tuple_struct(__deserializer, #type_name, #field_count, #visitor_expr)
            }
        }
        TupleForm::ExternallyTagged(_) => quote! {
            _serde::de::VariantAccess::tuple_variant(__variant, #field_count, #visitor_expr)
        },
        TupleForm::Untagged(_) => quote! {
            _serde::Deserializer::deserialize_tuple(__deserializer, #field_count, #visitor_expr)
        },
    };

    let visitor_var = if field_count == 0 {
        quote!(_)
    } else {
        quote!(mut __seq)
    };

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

            #visit_newtype_struct

            #[inline]
            fn visit_seq<__A>(self, #visitor_var: __A) -> _serde::#private::Result<Self::Value, __A::Error>
            where
                __A: _serde::de::SeqAccess<#delife>,
            {
                #visit_seq
            }
        }

        #dispatch
    }
}

/// Generates `Deserialize::deserialize_in_place` body for a `struct Tuple(...);` including `struct Newtype(T);`
#[cfg(feature = "deserialize_in_place")]
pub(super) fn deserialize_in_place(
    params: &Parameters,
    fields: &[Field],
    cattrs: &attr::Container,
) -> Fragment {
    assert!(
        !has_flatten(fields),
        "tuples and tuple variants cannot have flatten fields"
    );

    let field_count = fields
        .iter()
        .filter(|field| !field.attrs.skip_deserializing())
        .count();

    let this_type = &params.this_type;
    let (de_impl_generics, de_ty_generics, ty_generics, where_clause) =
        params.generics_with_de_lifetime();
    let delife = params.borrowed.de_lifetime();

    let expecting = format!("tuple struct {}", params.type_name());
    let expecting = cattrs.expecting().unwrap_or(&expecting);

    let nfields = fields.len();

    let visit_newtype_struct = if field_count == 1 {
        // We deserialize newtype, so only one field is not skipped

        let index = fields
            .iter()
            .position(|field| !field.attrs.skip_deserializing())
            .map(Index::from)
            .unwrap();
        let mut deserialize = quote! {
            _serde::Deserialize::deserialize_in_place(__e, &mut self.place.#index)
        };
        // Deserialize and write defaults if at least one field is skipped,
        // otherwise only deserialize
        if nfields > 1 {
            let write_defaults = fields.iter().enumerate().filter_map(|(index, field)| {
                if field.attrs.skip_deserializing() {
                    let index = Index::from(index);
                    let default = Expr(expr_is_missing(field, cattrs));
                    return Some(quote!(self.place.#index = #default;));
                }
                None
            });
            deserialize = quote! {
                match #deserialize {
                    _serde::#private::Ok(_) => {
                        #(#write_defaults)*
                        _serde::#private::Ok(())
                    }
                    _serde::#private::Err(__err) => _serde::#private::Err(__err),
                }
            }
        }

        Some(quote! {
            #[inline]
            fn visit_newtype_struct<__E>(self, __e: __E) -> _serde::#private::Result<Self::Value, __E::Error>
            where
                __E: _serde::Deserializer<#delife>,
            {
                #deserialize
            }
        })
    } else {
        None
    };

    let visit_seq = Stmts(read_fields_in_order_in_place(
        params, fields, cattrs, expecting,
    ));

    let visitor_expr = quote! {
        __Visitor {
            place: __place,
            lifetime: _serde::#private::PhantomData,
        }
    };

    let type_name = cattrs.name().deserialize_name();
    let dispatch = if field_count != 0 && nfields == 1 {
        quote!(_serde::Deserializer::deserialize_newtype_struct(__deserializer, #type_name, #visitor_expr))
    } else {
        quote!(_serde::Deserializer::deserialize_tuple_struct(__deserializer, #type_name, #field_count, #visitor_expr))
    };

    let visitor_var = if field_count == 0 {
        quote!(_)
    } else {
        quote!(mut __seq)
    };

    let in_place_impl_generics = de_impl_generics.in_place();
    let in_place_ty_generics = de_ty_generics.in_place();
    let place_life = place_lifetime();

    quote_block! {
        #[doc(hidden)]
        struct __Visitor #in_place_impl_generics #where_clause {
            place: &#place_life mut #this_type #ty_generics,
            lifetime: _serde::#private::PhantomData<&#delife ()>,
        }

        #[automatically_derived]
        impl #in_place_impl_generics _serde::de::Visitor<#delife> for __Visitor #in_place_ty_generics #where_clause {
            type Value = ();

            fn expecting(&self, __formatter: &mut _serde::#private::Formatter) -> _serde::#private::fmt::Result {
                _serde::#private::Formatter::write_str(__formatter, #expecting)
            }

            #visit_newtype_struct

            #[inline]
            fn visit_seq<__A>(self, #visitor_var: __A) -> _serde::#private::Result<Self::Value, __A::Error>
            where
                __A: _serde::de::SeqAccess<#delife>,
            {
                #visit_seq
            }
        }

        #dispatch
    }
}
