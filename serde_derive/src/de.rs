use syn::{self, Ident};
use quote::{self, Tokens};

use bound;
use fragment::{Fragment, Expr, Stmts, Match};
use internals::ast::{Body, Field, Item, Style, Variant};
use internals::{self, attr};

pub fn expand_derive_deserialize(item: &syn::DeriveInput) -> Result<Tokens, String> {
    let item = {
        let ctxt = internals::Ctxt::new();
        let item = Item::from_ast(&ctxt, item);
        check_no_str(&ctxt, &item);
        try!(ctxt.check());
        item
    };

    let ident = &item.ident;
    let generics = build_generics(&item);
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    let dummy_const = Ident::new(format!("_IMPL_DESERIALIZE_FOR_{}", ident));
    let body = Stmts(deserialize_body(&item, &generics));

    Ok(quote! {
        #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
        const #dummy_const: () = {
            extern crate serde as _serde;
            #[automatically_derived]
            impl #impl_generics _serde::Deserialize for #ident #ty_generics #where_clause {
                fn deserialize<__D>(deserializer: __D) -> _serde::export::Result<Self, __D::Error>
                    where __D: _serde::Deserializer
                {
                    #body
                }
            }
        };
    })
}

// All the generics in the input, plus a bound `T: Deserialize` for each generic
// field type that will be deserialized by us, plus a bound `T: Default` for
// each generic field type that will be set to a default value.
fn build_generics(item: &Item) -> syn::Generics {
    let generics = bound::without_defaults(item.generics);

    let generics =
        bound::with_where_predicates_from_fields(item, &generics, attr::Field::de_bound);

    match item.attrs.de_bound() {
        Some(predicates) => bound::with_where_predicates(&generics, predicates),
        None => {
            let generics = match *item.attrs.default() {
                attr::Default::Default => {
                    bound::with_self_bound(item, &generics, &path!(_serde::export::Default))
                }
                attr::Default::None | attr::Default::Path(_) => generics,
            };

            let generics =
                bound::with_bound(item,
                                  &generics,
                                  needs_deserialize_bound,
                                  &path!(_serde::Deserialize));

            bound::with_bound(item,
                              &generics,
                              requires_default,
                              &path!(_serde::export::Default))
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

fn deserialize_body(item: &Item, generics: &syn::Generics) -> Fragment {
    match item.body {
        Body::Enum(ref variants) => {
            deserialize_item_enum(&item.ident, generics, variants, &item.attrs)
        }
        Body::Struct(Style::Struct, ref fields) => {
            if fields.iter().any(|field| field.ident.is_none()) {
                panic!("struct has unnamed fields");
            }

            deserialize_struct(&item.ident,
                               None,
                               generics,
                               fields,
                               &item.attrs,
                               None)
        }
        Body::Struct(Style::Tuple, ref fields) |
        Body::Struct(Style::Newtype, ref fields) => {
            if fields.iter().any(|field| field.ident.is_some()) {
                panic!("tuple struct has named fields");
            }

            deserialize_tuple(&item.ident,
                              None,
                              generics,
                              fields,
                              &item.attrs,
                              None)
        }
        Body::Struct(Style::Unit, _) => deserialize_unit_struct(&item.ident, &item.attrs),
    }
}

fn deserialize_unit_struct(ident: &syn::Ident, item_attrs: &attr::Item) -> Fragment {
    let type_name = item_attrs.name().deserialize_name();

    let expecting = format!("unit struct {}", ident);

    quote_block! {
        struct __Visitor;

        impl _serde::de::Visitor for __Visitor {
            type Value = #ident;

            fn expecting(&self, formatter: &mut _serde::export::fmt::Formatter) -> _serde::export::fmt::Result {
                _serde::export::fmt::Formatter::write_str(formatter, #expecting)
            }

            #[inline]
            fn visit_unit<__E>(self) -> _serde::export::Result<#ident, __E>
                where __E: _serde::de::Error,
            {
                _serde::export::Ok(#ident)
            }

            #[inline]
            fn visit_seq<__V>(self, _: __V) -> _serde::export::Result<#ident, __V::Error>
                where __V: _serde::de::SeqVisitor,
            {
                _serde::export::Ok(#ident)
            }
        }

        deserializer.deserialize_unit_struct(#type_name, __Visitor)
    }
}

fn deserialize_tuple(ident: &syn::Ident,
                     variant_ident: Option<&syn::Ident>,
                     generics: &syn::Generics,
                     fields: &[Field],
                     item_attrs: &attr::Item,
                     deserializer: Option<Tokens>)
                     -> Fragment {
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let is_enum = variant_ident.is_some();
    let type_path = match variant_ident {
        Some(variant_ident) => quote!(#ident::#variant_ident),
        None => quote!(#ident),
    };
    let expecting = match variant_ident {
        Some(variant_ident) => format!("tuple variant {}::{}", ident, variant_ident),
        None => format!("tuple struct {}", ident),
    };

    let nfields = fields.len();

    let visit_newtype_struct = if !is_enum && nfields == 1 {
        Some(deserialize_newtype_struct(ident, &type_path, generics, &fields[0]))
    } else {
        None
    };

    let visit_seq = Stmts(deserialize_seq(ident, &type_path, generics, fields, false, item_attrs));

    let visitor_expr = quote! {
        __Visitor { marker: _serde::export::PhantomData::<#ident #ty_generics> }
    };
    let dispatch = if let Some(deserializer) = deserializer {
        quote!(_serde::Deserializer::deserialize_tuple(#deserializer, #nfields, #visitor_expr))
    } else if is_enum {
        quote!(_serde::de::VariantVisitor::visit_tuple(visitor, #nfields, #visitor_expr))
    } else if nfields == 1 {
        let type_name = item_attrs.name().deserialize_name();
        quote!(_serde::Deserializer::deserialize_newtype_struct(deserializer, #type_name, #visitor_expr))
    } else {
        let type_name = item_attrs.name().deserialize_name();
        quote!(_serde::Deserializer::deserialize_tuple_struct(deserializer, #type_name, #nfields, #visitor_expr))
    };

    let all_skipped = fields.iter().all(|field| field.attrs.skip_deserializing());
    let visitor_var = if all_skipped {
        quote!(_)
    } else {
        quote!(mut visitor)
    };

    quote_block! {
        struct __Visitor #impl_generics #where_clause {
            marker: _serde::export::PhantomData<#ident #ty_generics>,
        }

        impl #impl_generics _serde::de::Visitor for __Visitor #ty_generics #where_clause {
            type Value = #ident #ty_generics;

            fn expecting(&self, formatter: &mut _serde::export::fmt::Formatter) -> _serde::export::fmt::Result {
                _serde::export::fmt::Formatter::write_str(formatter, #expecting)
            }

            #visit_newtype_struct

            #[inline]
            fn visit_seq<__V>(self, #visitor_var: __V) -> _serde::export::Result<Self::Value, __V::Error>
                where __V: _serde::de::SeqVisitor
            {
                #visit_seq
            }
        }

        #dispatch
    }
}

fn deserialize_seq(ident: &syn::Ident,
                   type_path: &Tokens,
                   generics: &syn::Generics,
                   fields: &[Field],
                   is_struct: bool,
                   item_attrs: &attr::Item)
                   -> Fragment {
    let vars = (0..fields.len()).map(field_i as fn(_) -> _);

    let deserialized_count = fields.iter()
        .filter(|field| !field.attrs.skip_deserializing())
        .count();
    let expecting = format!("tuple of {} elements", deserialized_count);

    let mut index_in_seq = 0usize;
    let let_values = vars.clone().zip(fields)
        .map(|(var, field)| {
            if field.attrs.skip_deserializing() {
                let default = Expr(expr_is_missing(&field, item_attrs));
                quote! {
                    let #var = #default;
                }
            } else {
                let visit = match field.attrs.deserialize_with() {
                    None => {
                        let field_ty = &field.ty;
                        quote!(try!(_serde::de::SeqVisitor::visit::<#field_ty>(&mut visitor)))
                    }
                    Some(path) => {
                        let (wrapper, wrapper_ty) = wrap_deserialize_with(
                            ident, generics, field.ty, path);
                        quote!({
                            #wrapper
                            try!(_serde::de::SeqVisitor::visit::<#wrapper_ty>(&mut visitor))
                                .map(|wrap| wrap.value)
                        })
                    }
                };
                let assign = quote! {
                    let #var = match #visit {
                        Some(value) => { value },
                        None => {
                            return _serde::export::Err(_serde::de::Error::invalid_length(#index_in_seq, &#expecting));
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

    quote_block! {
        #(#let_values)*
        _serde::export::Ok(#result)
    }
}

fn deserialize_newtype_struct(ident: &syn::Ident,
                              type_path: &Tokens,
                              generics: &syn::Generics,
                              field: &Field)
                              -> Tokens {
    let value = match field.attrs.deserialize_with() {
        None => {
            let field_ty = &field.ty;
            quote! {
                try!(<#field_ty as _serde::Deserialize>::deserialize(__e))
            }
        }
        Some(path) => {
            let (wrapper, wrapper_ty) =
                wrap_deserialize_with(ident, generics, field.ty, path);
            quote!({
                #wrapper
                try!(<#wrapper_ty as _serde::Deserialize>::deserialize(__e)).value
            })
        }
    };
    quote! {
        #[inline]
        fn visit_newtype_struct<__E>(self, __e: __E) -> _serde::export::Result<Self::Value, __E::Error>
            where __E: _serde::Deserializer,
        {
            _serde::export::Ok(#type_path(#value))
        }
    }
}

fn deserialize_struct(ident: &syn::Ident,
                      variant_ident: Option<&syn::Ident>,
                      generics: &syn::Generics,
                      fields: &[Field],
                      item_attrs: &attr::Item,
                      deserializer: Option<Tokens>)
                      -> Fragment {
    let is_enum = variant_ident.is_some();
    let is_untagged = deserializer.is_some();

    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let type_path = match variant_ident {
        Some(variant_ident) => quote!(#ident::#variant_ident),
        None => quote!(#ident),
    };
    let expecting = match variant_ident {
        Some(variant_ident) => format!("struct variant {}::{}", ident, variant_ident),
        None => format!("struct {}", ident),
    };

    let visit_seq = Stmts(deserialize_seq(ident, &type_path, generics, fields, true, item_attrs));

    let (field_visitor, fields_stmt, visit_map) =
        deserialize_struct_visitor(ident, type_path, generics, fields, item_attrs);
    let field_visitor = Stmts(field_visitor);
    let fields_stmt = Stmts(fields_stmt);
    let visit_map = Stmts(visit_map);

    let visitor_expr = quote! {
        __Visitor { marker: _serde::export::PhantomData::<#ident #ty_generics> }
    };
    let dispatch = if let Some(deserializer) = deserializer {
        quote! {
            _serde::Deserializer::deserialize(#deserializer, #visitor_expr)
        }
    } else if is_enum {
        quote! {
            _serde::de::VariantVisitor::visit_struct(visitor, FIELDS, #visitor_expr)
        }
    } else {
        let type_name = item_attrs.name().deserialize_name();
        quote! {
            _serde::Deserializer::deserialize_struct(deserializer, #type_name, FIELDS, #visitor_expr)
        }
    };

    let all_skipped = fields.iter().all(|field| field.attrs.skip_deserializing());
    let visitor_var = if all_skipped {
        quote!(_)
    } else {
        quote!(mut visitor)
    };

    let visit_seq = if is_untagged {
        // untagged struct variants do not get a visit_seq method
        None
    } else {
        Some(quote! {
            #[inline]
            fn visit_seq<__V>(self, #visitor_var: __V) -> _serde::export::Result<Self::Value, __V::Error>
                where __V: _serde::de::SeqVisitor
            {
                #visit_seq
            }
        })
    };

    quote_block! {
        #field_visitor

        struct __Visitor #impl_generics #where_clause {
            marker: _serde::export::PhantomData<#ident #ty_generics>,
        }

        impl #generics _serde::de::Visitor for __Visitor #ty_generics #where_clause {
            type Value = #ident #ty_generics;

            fn expecting(&self, formatter: &mut _serde::export::fmt::Formatter) -> _serde::export::fmt::Result {
                _serde::export::fmt::Formatter::write_str(formatter, #expecting)
            }

            #visit_seq

            #[inline]
            fn visit_map<__V>(self, mut visitor: __V) -> _serde::export::Result<Self::Value, __V::Error>
                where __V: _serde::de::MapVisitor
            {
                #visit_map
            }
        }

        #fields_stmt

        #dispatch
    }
}

fn deserialize_item_enum(ident: &syn::Ident,
                         generics: &syn::Generics,
                         variants: &[Variant],
                         item_attrs: &attr::Item)
                         -> Fragment {
    match *item_attrs.tag() {
        attr::EnumTag::External => {
            deserialize_externally_tagged_enum(ident, generics, variants, item_attrs)
        }
        attr::EnumTag::Internal { ref tag } => {
            deserialize_internally_tagged_enum(ident,
                                               generics,
                                               variants,
                                               item_attrs,
                                               tag)
        }
        attr::EnumTag::Adjacent { ref tag, ref content } => {
            deserialize_adjacently_tagged_enum(ident,
                                               generics,
                                               variants,
                                               item_attrs,
                                               tag,
                                               content)
        }
        attr::EnumTag::None => {
            deserialize_untagged_enum(ident, generics, variants, item_attrs)
        }
    }
}

fn deserialize_externally_tagged_enum(ident: &syn::Ident,
                                      generics: &syn::Generics,
                                      variants: &[Variant],
                                      item_attrs: &attr::Item)
                                      -> Fragment {
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let type_name = item_attrs.name().deserialize_name();

    let expecting = format!("enum {}", ident);

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

    let variant_visitor = Stmts(deserialize_field_visitor(variant_names_idents, item_attrs, true));

    // Match arms to extract a variant from a string
    let variant_arms = variants.iter()
        .enumerate()
        .filter(|&(_, variant)| !variant.attrs.skip_deserializing())
        .map(|(i, variant)| {
            let variant_name = field_i(i);

            let block = Match(deserialize_externally_tagged_variant(ident,
                                                                    generics,
                                                                    variant,
                                                                    item_attrs));

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
            // let _serde::export::Err(err) = _serde::de::EnumVisitor::visit_variant::<__Field>(visitor);
            // _serde::export::Err(err)
            _serde::de::EnumVisitor::visit_variant::<__Field>(visitor)
                .map(|(impossible, _)| match impossible {})
        }
    } else {
        quote! {
            match try!(_serde::de::EnumVisitor::visit_variant(visitor)) {
                #(#variant_arms)*
            }
        }
    };

    quote_block! {
        #variant_visitor

        struct __Visitor #impl_generics #where_clause {
            marker: _serde::export::PhantomData<#ident #ty_generics>,
        }

        impl #generics _serde::de::Visitor for __Visitor #ty_generics #where_clause {
            type Value = #ident #ty_generics;

            fn expecting(&self, formatter: &mut _serde::export::fmt::Formatter) -> _serde::export::fmt::Result {
                _serde::export::fmt::Formatter::write_str(formatter, #expecting)
            }

            fn visit_enum<__V>(self, visitor: __V) -> _serde::export::Result<Self::Value, __V::Error>
                where __V: _serde::de::EnumVisitor,
            {
                #match_variant
            }
        }

        #variants_stmt

        _serde::Deserializer::deserialize_enum(deserializer, #type_name, VARIANTS,
                                               __Visitor {
                                                   marker: _serde::export::PhantomData::<#ident #ty_generics>,
                                               })
    }
}

fn deserialize_internally_tagged_enum(ident: &syn::Ident,
                                      generics: &syn::Generics,
                                      variants: &[Variant],
                                      item_attrs: &attr::Item,
                                      tag: &str)
                                      -> Fragment {
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

    let variant_visitor = Stmts(deserialize_field_visitor(variant_names_idents, item_attrs, true));

    // Match arms to extract a variant from a string
    let variant_arms = variants.iter()
        .enumerate()
        .filter(|&(_, variant)| !variant.attrs.skip_deserializing())
        .map(|(i, variant)| {
            let variant_name = field_i(i);

            let block = Match(deserialize_internally_tagged_variant(
                ident,
                generics,
                variant,
                item_attrs,
                quote!(_serde::de::private::ContentDeserializer::<__D::Error>::new(_tagged.content)),
            ));

            quote! {
                __Field::#variant_name => #block
            }
        });

    quote_block! {
        #variant_visitor

        #variants_stmt

        let _tagged = try!(_serde::Deserializer::deserialize(
            deserializer,
            _serde::de::private::TaggedContentVisitor::<__Field>::new(#tag)));

        match _tagged.tag {
            #(#variant_arms)*
        }
    }
}

fn deserialize_adjacently_tagged_enum(ident: &syn::Ident,
                                      generics: &syn::Generics,
                                      variants: &[Variant],
                                      item_attrs: &attr::Item,
                                      tag: &str,
                                      content: &str)
                                      -> Fragment {
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

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

    let variant_visitor = Stmts(deserialize_field_visitor(variant_names_idents, item_attrs, true));

    let ref variant_arms: Vec<_> = variants.iter()
        .enumerate()
        .filter(|&(_, variant)| !variant.attrs.skip_deserializing())
        .map(|(i, variant)| {
            let variant_index = field_i(i);

            let block = Match(deserialize_untagged_variant(
                ident,
                generics,
                variant,
                item_attrs,
                quote!(_deserializer),
            ));

            quote! {
                __Field::#variant_index => #block
            }
        })
        .collect();

    let expecting = format!("adjacently tagged enum {}", ident);
    let type_name = item_attrs.name().deserialize_name();

    let tag_or_content = quote! {
        _serde::de::private::TagOrContentFieldVisitor {
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
        _serde::export::Err(<__V::Error as _serde::de::Error>::missing_field(#content))
    };
    if variants.iter().any(is_unit) {
        let fallthrough = if variants.iter().all(is_unit) {
            None
        } else {
            Some(quote! {
                _ => #missing_content
            })
        };
        let arms = variants.iter()
            .enumerate()
            .filter(|&(_, variant)| !variant.attrs.skip_deserializing() && is_unit(variant))
            .map(|(i, variant)| {
                let variant_index = field_i(i);
                let variant_ident = &variant.ident;
                quote! {
                    __Field::#variant_index => _serde::export::Ok(#ident::#variant_ident),
                }
            });
        missing_content = quote! {
            match __field {
                #(#arms)*
                #fallthrough
            }
        };
    }

    let visit_third_key = quote! {
        // Visit the third key in the map, hopefully there isn't one.
        match try!(_serde::de::MapVisitor::visit_key_seed(&mut visitor, #tag_or_content)) {
            _serde::export::Some(_serde::de::private::TagOrContentField::Tag) => {
                _serde::export::Err(<__V::Error as _serde::de::Error>::duplicate_field(#tag))
            }
            _serde::export::Some(_serde::de::private::TagOrContentField::Content) => {
                _serde::export::Err(<__V::Error as _serde::de::Error>::duplicate_field(#content))
            }
            _serde::export::None => _serde::export::Ok(__ret),
        }
    };

    quote_block! {
        #variant_visitor

        #variants_stmt

        struct __Seed #impl_generics #where_clause {
            field: __Field,
            marker: _serde::export::PhantomData<#ident #ty_generics>,
        }

        impl #impl_generics _serde::de::DeserializeSeed for __Seed #ty_generics #where_clause {
            type Value = #ident #ty_generics;

            fn deserialize<__D>(self, _deserializer: __D) -> _serde::export::Result<Self::Value, __D::Error>
                where __D: _serde::Deserializer
            {
                match self.field {
                    #(#variant_arms)*
                }
            }
        }

        struct __Visitor #impl_generics #where_clause {
            marker: _serde::export::PhantomData<#ident #ty_generics>,
        }

        impl #impl_generics _serde::de::Visitor for __Visitor #ty_generics #where_clause {
            type Value = #ident #ty_generics;

            fn expecting(&self, formatter: &mut _serde::export::fmt::Formatter) -> _serde::export::fmt::Result {
                _serde::export::fmt::Formatter::write_str(formatter, #expecting)
            }

            fn visit_map<__V>(self, mut visitor: __V) -> _serde::export::Result<Self::Value, __V::Error>
                where __V: _serde::de::MapVisitor
            {
                // Visit the first key.
                match try!(_serde::de::MapVisitor::visit_key_seed(&mut visitor, #tag_or_content)) {
                    // First key is the tag.
                    _serde::export::Some(_serde::de::private::TagOrContentField::Tag) => {
                        // Parse the tag.
                        let __field = try!(_serde::de::MapVisitor::visit_value(&mut visitor));
                        // Visit the second key.
                        match try!(_serde::de::MapVisitor::visit_key_seed(&mut visitor, #tag_or_content)) {
                            // Second key is a duplicate of the tag.
                            _serde::export::Some(_serde::de::private::TagOrContentField::Tag) => {
                                _serde::export::Err(<__V::Error as _serde::de::Error>::duplicate_field(#tag))
                            }
                            // Second key is the content.
                            _serde::export::Some(_serde::de::private::TagOrContentField::Content) => {
                                let __ret = try!(_serde::de::MapVisitor::visit_value_seed(&mut visitor, __Seed { field: __field, marker: _serde::export::PhantomData }));
                                // Visit the third key, hopefully there isn't one.
                                #visit_third_key
                            }
                            // There is no second key; might be okay if the we have a unit variant.
                            _serde::export::None => #missing_content
                        }
                    }
                    // First key is the content.
                    _serde::export::Some(_serde::de::private::TagOrContentField::Content) => {
                        // Buffer up the content.
                        let __content = try!(_serde::de::MapVisitor::visit_value::<_serde::de::private::Content>(&mut visitor));
                        // Visit the second key.
                        match try!(_serde::de::MapVisitor::visit_key_seed(&mut visitor, #tag_or_content)) {
                            // Second key is the tag.
                            _serde::export::Some(_serde::de::private::TagOrContentField::Tag) => {
                                let _deserializer = _serde::de::private::ContentDeserializer::<__V::Error>::new(__content);
                                // Parse the tag.
                                let __ret = try!(match try!(_serde::de::MapVisitor::visit_value(&mut visitor)) {
                                    // Deserialize the buffered content now that we know the variant.
                                    #(#variant_arms)*
                                });
                                // Visit the third key, hopefully there isn't one.
                                #visit_third_key
                            }
                            // Second key is a duplicate of the content.
                            _serde::export::Some(_serde::de::private::TagOrContentField::Content) => {
                                _serde::export::Err(<__V::Error as _serde::de::Error>::duplicate_field(#content))
                            }
                            // There is no second key.
                            _serde::export::None => {
                                _serde::export::Err(<__V::Error as _serde::de::Error>::missing_field(#tag))
                            }
                        }
                    }
                    // There is no first key.
                    _serde::export::None => {
                        _serde::export::Err(<__V::Error as _serde::de::Error>::missing_field(#tag))
                    }
                }
            }

            fn visit_seq<__V>(self, mut visitor: __V) -> _serde::export::Result<Self::Value, __V::Error>
                where __V: _serde::de::SeqVisitor
            {
                // Visit the first element - the tag.
                match try!(_serde::de::SeqVisitor::visit(&mut visitor)) {
                    _serde::export::Some(__field) => {
                        // Visit the second element - the content.
                        match try!(_serde::de::SeqVisitor::visit_seed(&mut visitor, __Seed { field: __field, marker: _serde::export::PhantomData })) {
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
        _serde::Deserializer::deserialize_struct(deserializer, #type_name, FIELDS,
            __Visitor { marker: _serde::export::PhantomData::<#ident #ty_generics> })
    }
}

fn deserialize_untagged_enum(ident: &syn::Ident,
                             generics: &syn::Generics,
                             variants: &[Variant],
                             item_attrs: &attr::Item)
                             -> Fragment {
    let attempts = variants.iter()
        .filter(|variant| !variant.attrs.skip_deserializing())
        .map(|variant| {
            Expr(deserialize_untagged_variant(
                ident,
                generics,
                variant,
                item_attrs,
                quote!(_serde::de::private::ContentRefDeserializer::<__D::Error>::new(&_content)),
            ))
        });

    // TODO this message could be better by saving the errors from the failed
    // attempts. The heuristic used by TOML was to count the number of fields
    // processed before an error, and use the error that happened after the
    // largest number of fields. I'm not sure I like that. Maybe it would be
    // better to save all the errors and combine them into one message that
    // explains why none of the variants matched.
    let fallthrough_msg = format!("data did not match any variant of untagged enum {}", ident);

    quote_block! {
        let _content = try!(<_serde::de::private::Content as _serde::Deserialize>::deserialize(deserializer));

        #(
            if let _serde::export::Ok(ok) = #attempts {
                return _serde::export::Ok(ok);
            }
        )*

        _serde::export::Err(_serde::de::Error::custom(#fallthrough_msg))
    }
}

fn deserialize_externally_tagged_variant(ident: &syn::Ident,
                                         generics: &syn::Generics,
                                         variant: &Variant,
                                         item_attrs: &attr::Item)
                                         -> Fragment {
    let variant_ident = &variant.ident;

    match variant.style {
        Style::Unit => {
            quote_block! {
                try!(_serde::de::VariantVisitor::visit_unit(visitor));
                _serde::export::Ok(#ident::#variant_ident)
            }
        }
        Style::Newtype => {
            deserialize_externally_tagged_newtype_variant(ident,
                                                          variant_ident,
                                                          generics,
                                                          &variant.fields[0])
        }
        Style::Tuple => {
            deserialize_tuple(ident,
                              Some(variant_ident),
                              generics,
                              &variant.fields,
                              item_attrs,
                              None)
        }
        Style::Struct => {
            deserialize_struct(ident,
                               Some(variant_ident),
                               generics,
                               &variant.fields,
                               item_attrs,
                               None)
        }
    }
}

fn deserialize_internally_tagged_variant(ident: &syn::Ident,
                                         generics: &syn::Generics,
                                         variant: &Variant,
                                         item_attrs: &attr::Item,
                                         deserializer: Tokens)
                                         -> Fragment {
    let variant_ident = &variant.ident;

    match variant.style {
        Style::Unit => {
            let type_name = ident.as_ref();
            let variant_name = variant.ident.as_ref();
            quote_block! {
                try!(_serde::Deserializer::deserialize(#deserializer, _serde::de::private::InternallyTaggedUnitVisitor::new(#type_name, #variant_name)));
                _serde::export::Ok(#ident::#variant_ident)
            }
        }
        Style::Newtype | Style::Struct => {
            deserialize_untagged_variant(ident,
                                         generics,
                                         variant,
                                         item_attrs,
                                         deserializer)
        }
        Style::Tuple => unreachable!("checked in serde_codegen_internals"),
    }
}

fn deserialize_untagged_variant(ident: &syn::Ident,
                                generics: &syn::Generics,
                                variant: &Variant,
                                item_attrs: &attr::Item,
                                deserializer: Tokens)
                                -> Fragment {
    let variant_ident = &variant.ident;

    match variant.style {
        Style::Unit => {
            let type_name = ident.as_ref();
            let variant_name = variant.ident.as_ref();
            quote_expr! {
                _serde::export::Result::map(
                    _serde::Deserializer::deserialize(
                        #deserializer,
                        _serde::de::private::UntaggedUnitVisitor::new(#type_name, #variant_name)
                    ),
                    |()| #ident::#variant_ident)
            }
        }
        Style::Newtype => {
            deserialize_untagged_newtype_variant(ident,
                                                 variant_ident,
                                                 generics,
                                                 &variant.fields[0],
                                                 deserializer)
        }
        Style::Tuple => {
            deserialize_tuple(ident,
                              Some(variant_ident),
                              generics,
                              &variant.fields,
                              item_attrs,
                              Some(deserializer))
        }
        Style::Struct => {
            deserialize_struct(ident,
                               Some(variant_ident),
                               generics,
                               &variant.fields,
                               item_attrs,
                               Some(deserializer))
        }
    }
}

fn deserialize_externally_tagged_newtype_variant(ident: &syn::Ident,
                                                 variant_ident: &syn::Ident,
                                                 generics: &syn::Generics,
                                                 field: &Field)
                                                 -> Fragment {
    match field.attrs.deserialize_with() {
        None => {
            let field_ty = &field.ty;
            quote_expr! {
                _serde::export::Result::map(
                    _serde::de::VariantVisitor::visit_newtype::<#field_ty>(visitor),
                    #ident::#variant_ident)
            }
        }
        Some(path) => {
            let (wrapper, wrapper_ty) =
                wrap_deserialize_with(ident, generics, field.ty, path);
            quote_block! {
                #wrapper
                _serde::export::Result::map(
                    _serde::de::VariantVisitor::visit_newtype::<#wrapper_ty>(visitor),
                    |_wrapper| #ident::#variant_ident(_wrapper.value))
            }
        }
    }
}

fn deserialize_untagged_newtype_variant(ident: &syn::Ident,
                                        variant_ident: &syn::Ident,
                                        generics: &syn::Generics,
                                        field: &Field,
                                        deserializer: Tokens)
                                        -> Fragment {
    match field.attrs.deserialize_with() {
        None => {
            let field_ty = &field.ty;
            quote_expr! {
                _serde::export::Result::map(
                    <#field_ty as _serde::Deserialize>::deserialize(#deserializer),
                    #ident::#variant_ident)
            }
        }
        Some(path) => {
            let (wrapper, wrapper_ty) =
                wrap_deserialize_with(ident, generics, field.ty, path);
            quote_block! {
                #wrapper
                _serde::export::Result::map(
                    <#wrapper_ty as _serde::Deserialize>::deserialize(#deserializer),
                    |_wrapper| #ident::#variant_ident(_wrapper.value))
            }
        }
    }
}

fn deserialize_field_visitor(fields: Vec<(String, Ident)>,
                             item_attrs: &attr::Item,
                             is_variant: bool)
                             -> Fragment {
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
                        #variant_indices => _serde::export::Ok(__Field::#field_idents),
                    )*
                    _ => _serde::export::Err(_serde::de::Error::invalid_value(
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
            _serde::export::Err(_serde::de::Error::unknown_variant(value, VARIANTS))
        }
    } else if item_attrs.deny_unknown_fields() {
        quote! {
            _serde::export::Err(_serde::de::Error::unknown_field(value, FIELDS))
        }
    } else {
        quote! {
            _serde::export::Ok(__Field::__ignore)
        }
    };

    let bytes_to_str = if is_variant || item_attrs.deny_unknown_fields() {
        Some(quote! {
            let value = &_serde::export::from_utf8_lossy(value);
        })
    } else {
        None
    };

    quote_block! {
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
                        _serde::export::fmt::Formatter::write_str(formatter, "field name")
                    }

                    #visit_index

                    fn visit_str<__E>(self, value: &str) -> _serde::export::Result<__Field, __E>
                        where __E: _serde::de::Error
                    {
                        match value {
                            #(
                                #field_strs => _serde::export::Ok(__Field::#field_idents),
                            )*
                            _ => #fallthrough_arm
                        }
                    }

                    fn visit_bytes<__E>(self, value: &[u8]) -> _serde::export::Result<__Field, __E>
                        where __E: _serde::de::Error
                    {
                        match value {
                            #(
                                #field_bytes => _serde::export::Ok(__Field::#field_idents),
                            )*
                            _ => {
                                #bytes_to_str
                                #fallthrough_arm
                            }
                        }
                    }
                }

                _serde::Deserializer::deserialize_struct_field(deserializer, __FieldVisitor)
            }
        }
    }
}

fn deserialize_struct_visitor(ident: &syn::Ident,
                              struct_path: Tokens,
                              generics: &syn::Generics,
                              fields: &[Field],
                              item_attrs: &attr::Item)
                              -> (Fragment, Fragment, Fragment) {
    let field_names_idents: Vec<_> = fields.iter()
        .enumerate()
        .filter(|&(_, field)| !field.attrs.skip_deserializing())
        .map(|(i, field)| (field.attrs.name().deserialize_name(), field_i(i)))
        .collect();

    let fields_stmt = {
        let field_names = field_names_idents.iter().map(|&(ref name, _)| name);
        quote_block! {
            const FIELDS: &'static [&'static str] = &[ #(#field_names),* ];
        }
    };

    let field_visitor = deserialize_field_visitor(field_names_idents, item_attrs, false);

    let visit_map = deserialize_map(ident, struct_path, generics, fields, item_attrs);

    (field_visitor, fields_stmt, visit_map)
}

fn deserialize_map(ident: &syn::Ident,
                   struct_path: Tokens,
                   generics: &syn::Generics,
                   fields: &[Field],
                   item_attrs: &attr::Item)
                   -> Fragment {
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
                let mut #name: _serde::export::Option<#field_ty> = _serde::export::None;
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
                        try!(_serde::de::MapVisitor::visit_value::<#field_ty>(&mut visitor))
                    }
                }
                Some(path) => {
                    let (wrapper, wrapper_ty) = wrap_deserialize_with(
                        ident, generics, field.ty, path);
                    quote!({
                        #wrapper
                        try!(_serde::de::MapVisitor::visit_value::<#wrapper_ty>(&mut visitor)).value
                    })
                }
            };
            quote! {
                __Field::#name => {
                    if _serde::export::Option::is_some(&#name) {
                        return _serde::export::Err(<__V::Error as _serde::de::Error>::duplicate_field(#deser_name));
                    }
                    #name = _serde::export::Some(#visit);
                }
            }
        });

    // Visit ignored values to consume them
    let ignored_arm = if item_attrs.deny_unknown_fields() {
        None
    } else {
        Some(quote! {
            _ => { let _ = try!(_serde::de::MapVisitor::visit_value::<_serde::de::impls::IgnoredAny>(&mut visitor)); }
        })
    };

    let all_skipped = fields.iter().all(|field| field.attrs.skip_deserializing());
    let match_keys = if item_attrs.deny_unknown_fields() && all_skipped {
        quote! {
            // FIXME: Once we drop support for Rust 1.15:
            // let _serde::export::None::<__Field> = try!(_serde::de::MapVisitor::visit_key(&mut visitor));
            try!(_serde::de::MapVisitor::visit_key::<__Field>(&mut visitor))
                .map(|impossible| match impossible {});
        }
    } else {
        quote! {
            while let _serde::export::Some(key) = try!(_serde::de::MapVisitor::visit_key::<__Field>(&mut visitor)) {
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
            let missing_expr = Match(expr_is_missing(&field, item_attrs));

            quote! {
                let #name = match #name {
                    _serde::export::Some(#name) => #name,
                    _serde::export::None => #missing_expr
                };
            }
        });

    let result = fields_names.iter()
        .map(|&(field, ref name)| {
            let ident = field.ident.clone().expect("struct contains unnamed fields");
            if field.attrs.skip_deserializing() {
                let value = Expr(expr_is_missing(&field, item_attrs));
                quote!(#ident: #value)
            } else {
                quote!(#ident: #name)
            }
        });

    let let_default = match *item_attrs.default() {
        attr::Default::Default => {
            Some(quote!(
                let __default: Self::Value = _serde::export::Default::default();
            ))
        }
        attr::Default::Path(ref path) => {
            Some(quote!(
                let __default: Self::Value = #path();
            ))
        }
        attr::Default::None => {
            // We don't need the default value, to prevent an unused variable warning
            // we'll leave the line empty.
            None
        }
    };

    quote_block! {
        #(#let_values)*

        #match_keys

        #let_default

        #(#extract_values)*

        _serde::export::Ok(#struct_path { #(#result),* })
    }
}

fn field_i(i: usize) -> Ident {
    Ident::new(format!("__field{}", i))
}

/// This function wraps the expression in `#[serde(deserialize_with="...")]` in
/// a trait to prevent it from accessing the internal `Deserialize` state.
fn wrap_deserialize_with(ident: &syn::Ident,
                         generics: &syn::Generics,
                         field_ty: &syn::Ty,
                         deserialize_with: &syn::Path)
                         -> (Tokens, Tokens) {
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let wrapper = quote! {
        struct __DeserializeWith #impl_generics #where_clause {
            value: #field_ty,
            phantom: _serde::export::PhantomData<#ident #ty_generics>,
        }

        impl #impl_generics _serde::Deserialize for __DeserializeWith #ty_generics #where_clause {
            fn deserialize<__D>(__d: __D) -> _serde::export::Result<Self, __D::Error>
                where __D: _serde::Deserializer
            {
                let value = try!(#deserialize_with(__d));
                _serde::export::Ok(__DeserializeWith {
                    value: value,
                    phantom: _serde::export::PhantomData,
                })
            }
        }
    };

    let wrapper_ty = quote!(__DeserializeWith #ty_generics);

    (wrapper, wrapper_ty)
}

fn expr_is_missing(field: &Field, item_attrs: &attr::Item) -> Fragment {
    match *field.attrs.default() {
        attr::Default::Default => {
            return quote_expr!(_serde::export::Default::default());
        }
        attr::Default::Path(ref path) => {
            return quote_expr!(#path());
        }
        attr::Default::None => { /* below */ }
    }

    match *item_attrs.default() {
        attr::Default::Default | attr::Default::Path(_) => {
            let ident = &field.ident;
            return quote_expr!(__default.#ident);
        }
        attr::Default::None => { /* below */ }
    }

    let name = field.attrs.name().deserialize_name();
    match field.attrs.deserialize_with() {
        None => {
            quote_expr! {
                try!(_serde::de::private::missing_field(#name))
            }
        }
        Some(_) => {
            quote_expr! {
                return _serde::export::Err(<__V::Error as _serde::de::Error>::missing_field(#name))
            }
        }
    }
}

fn check_no_str(cx: &internals::Ctxt, item: &Item) {
    let fail = || {
        cx.error("Serde does not support deserializing fields of type &str; consider using \
                  String instead");
    };

    for field in item.body.all_fields() {
        if field.attrs.skip_deserializing() || field.attrs.deserialize_with().is_some() {
            continue;
        }

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
