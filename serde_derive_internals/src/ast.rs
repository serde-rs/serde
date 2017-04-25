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

pub struct Container<'a> {
    pub ident: syn::Ident,
    pub attrs: attr::Container,
    pub body: Body<'a>,
    pub generics: &'a syn::Generics,
}

pub enum Body<'a> {
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
    pub ty: &'a syn::Ty,
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
        let attrs = attr::Container::from_ast(cx, item);

        let mut body = match item.body {
            syn::Body::Enum(ref variants) => Body::Enum(enum_from_ast(cx, variants)),
            syn::Body::Struct(ref variant_data) => {
                let (style, fields) = struct_from_ast(cx, variant_data);
                Body::Struct(style, fields)
            }
        };

        match body {
            Body::Enum(ref mut variants) => {
                for ref mut variant in variants {
                    variant.attrs.rename_by_rule(attrs.rename_all());
                    for ref mut field in &mut variant.fields {
                        field.attrs.rename_by_rule(variant.attrs.rename_all());
                    }
                }
            }
            Body::Struct(_, ref mut fields) => {
                for field in fields {
                    field.attrs.rename_by_rule(attrs.rename_all());
                }
            }
        }

        let item = Container {
            ident: item.ident.clone(),
            attrs: attrs,
            body: body,
            generics: &item.generics,
        };
        check::check(cx, &item);
        item
    }
}

impl<'a> Body<'a> {
    pub fn all_fields(&'a self) -> Box<Iterator<Item = &'a Field<'a>> + 'a> {
        match *self {
            Body::Enum(ref variants) => {
                Box::new(variants.iter().flat_map(|variant| variant.fields.iter()))
            }
            Body::Struct(_, ref fields) => Box::new(fields.iter()),
        }
    }

    pub fn has_getter(&self) -> bool {
        self.all_fields().any(|f| f.attrs.getter().is_some())
    }
}

fn enum_from_ast<'a>(cx: &Ctxt, variants: &'a [syn::Variant]) -> Vec<Variant<'a>> {
    variants
        .iter()
        .map(
            |variant| {
                let (style, fields) = struct_from_ast(cx, &variant.data);
                Variant {
                    ident: variant.ident.clone(),
                    attrs: attr::Variant::from_ast(cx, variant),
                    style: style,
                    fields: fields,
                }
            },
        )
        .collect()
}

fn struct_from_ast<'a>(cx: &Ctxt, data: &'a syn::VariantData) -> (Style, Vec<Field<'a>>) {
    match *data {
        syn::VariantData::Struct(ref fields) => (Style::Struct, fields_from_ast(cx, fields)),
        syn::VariantData::Tuple(ref fields) if fields.len() == 1 => {
            (Style::Newtype, fields_from_ast(cx, fields))
        }
        syn::VariantData::Tuple(ref fields) => (Style::Tuple, fields_from_ast(cx, fields)),
        syn::VariantData::Unit => (Style::Unit, Vec::new()),
    }
}

fn fields_from_ast<'a>(cx: &Ctxt, fields: &'a [syn::Field]) -> Vec<Field<'a>> {
    fields
        .iter()
        .enumerate()
        .map(
            |(i, field)| {
                Field {
                    ident: field.ident.clone(),
                    attrs: attr::Field::from_ast(cx, i, field),
                    ty: &field.ty,
                }
            },
        )
        .collect()
}
