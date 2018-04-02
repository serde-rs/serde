// Copyright 2017 Serde Developers
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use syn;
use attr;
use check;
use Ctxt;
use syn::punctuated::Punctuated;

pub struct Container<'a> {
    pub ident: syn::Ident,
    pub attrs: attr::Container,
    pub data: Data<'a>,
    pub generics: &'a syn::Generics,
}

pub enum Data<'a> {
    Enum(Vec<Variant<'a>>),
    Struct(Style, Vec<Field<'a>>),
}

pub struct Variant<'a> {
    pub ident: syn::Ident,
    pub attrs: attr::Variant,
    pub style: Style,
    pub fields: Vec<Field<'a>>,
}

pub struct Field<'a> {
    pub ident: Option<syn::Ident>,
    pub attrs: attr::Field,
    pub ty: &'a syn::Type,
    pub original: &'a syn::Field,
}

#[derive(Copy, Clone)]
pub enum Style {
    Struct,
    Tuple,
    Newtype,
    Unit,
}

impl<'a> Container<'a> {
    pub fn from_ast(cx: &Ctxt, item: &'a syn::DeriveInput) -> Container<'a> {
        let mut attrs = attr::Container::from_ast(cx, item);

        let mut data = match item.data {
            syn::Data::Enum(ref data) => {
                Data::Enum(enum_from_ast(cx, &data.variants, attrs.default()))
            }
            syn::Data::Struct(ref data) => {
                let (style, fields) = struct_from_ast(cx, &data.fields, None, attrs.default());
                Data::Struct(style, fields)
            }
            syn::Data::Union(_) => {
                panic!("Serde does not support derive for unions");
            }
        };

        let mut has_flatten = false;
        match data {
            Data::Enum(ref mut variants) => for variant in variants {
                variant.attrs.rename_by_rule(attrs.rename_all());
                for field in &mut variant.fields {
                    if field.attrs.flatten() {
                        has_flatten = true;
                    }
                    field.attrs.rename_by_rule(variant.attrs.rename_all());
                }
            },
            Data::Struct(_, ref mut fields) => for field in fields {
                if field.attrs.flatten() {
                    has_flatten = true;
                }
                field.attrs.rename_by_rule(attrs.rename_all());
            },
        }

        if has_flatten {
            attrs.mark_has_flatten();
        }

        let item = Container {
            ident: item.ident,
            attrs: attrs,
            data: data,
            generics: &item.generics,
        };
        check::check(cx, &item);
        item
    }
}

impl<'a> Data<'a> {
    pub fn all_fields(&'a self) -> Box<Iterator<Item = &'a Field<'a>> + 'a> {
        match *self {
            Data::Enum(ref variants) => {
                Box::new(variants.iter().flat_map(|variant| variant.fields.iter()))
            }
            Data::Struct(_, ref fields) => Box::new(fields.iter()),
        }
    }

    pub fn has_getter(&self) -> bool {
        self.all_fields().any(|f| f.attrs.getter().is_some())
    }
}

fn enum_from_ast<'a>(
    cx: &Ctxt,
    variants: &'a Punctuated<syn::Variant, Token![,]>,
    container_default: &attr::Default,
) -> Vec<Variant<'a>> {
    variants
        .iter()
        .map(|variant| {
            let attrs = attr::Variant::from_ast(cx, variant);
            let (style, fields) =
                struct_from_ast(cx, &variant.fields, Some(&attrs), container_default);
            Variant {
                ident: variant.ident,
                attrs: attrs,
                style: style,
                fields: fields,
            }
        })
        .collect()
}

fn struct_from_ast<'a>(
    cx: &Ctxt,
    fields: &'a syn::Fields,
    attrs: Option<&attr::Variant>,
    container_default: &attr::Default,
) -> (Style, Vec<Field<'a>>) {
    match *fields {
        syn::Fields::Named(ref fields) => (
            Style::Struct,
            fields_from_ast(cx, &fields.named, attrs, container_default),
        ),
        syn::Fields::Unnamed(ref fields) if fields.unnamed.len() == 1 => (
            Style::Newtype,
            fields_from_ast(cx, &fields.unnamed, attrs, container_default),
        ),
        syn::Fields::Unnamed(ref fields) => (
            Style::Tuple,
            fields_from_ast(cx, &fields.unnamed, attrs, container_default),
        ),
        syn::Fields::Unit => (Style::Unit, Vec::new()),
    }
}

fn fields_from_ast<'a>(
    cx: &Ctxt,
    fields: &'a Punctuated<syn::Field, Token![,]>,
    attrs: Option<&attr::Variant>,
    container_default: &attr::Default,
) -> Vec<Field<'a>> {
    fields
        .iter()
        .enumerate()
        .map(|(i, field)| Field {
            ident: field.ident,
            attrs: attr::Field::from_ast(cx, i, field, attrs, container_default),
            ty: &field.ty,
            original: field,
        })
        .collect()
}
