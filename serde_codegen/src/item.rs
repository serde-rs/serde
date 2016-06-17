use syntax::ast;
use syntax::codemap;
use syntax::ext::base::ExtCtxt;
use syntax::ptr::P;

use attr;
use error::Error;

pub struct Item<'a> {
    pub ident: ast::Ident,
    pub span: codemap::Span,
    pub attrs: attr::ContainerAttrs,
    pub body: Body<'a>,
    pub generics: &'a ast::Generics,
}

pub enum Body<'a> {
    Enum(Vec<Variant<'a>>),
    Struct(Style, Vec<Field<'a>>),
}

pub struct Variant<'a> {
    pub ident: ast::Ident,
    pub attrs: attr::VariantAttrs,
    pub style: Style,
    pub fields: Vec<Field<'a>>,
}

pub struct Field<'a> {
    pub ident: Option<ast::Ident>,
    pub span: codemap::Span,
    pub attrs: attr::FieldAttrs,
    pub ty: &'a P<ast::Ty>,
}

pub enum Style {
    Struct,
    Tuple,
    Newtype,
    Unit,
}

impl<'a> Item<'a> {
    pub fn from_ast(
        cx: &ExtCtxt,
        derive_trait: &'static str,
        item: &'a ast::Item,
    ) -> Result<Item<'a>, Error> {
        let attrs = attr::ContainerAttrs::from_item(cx, item);

        let (body, generics) = match item.node {
            ast::ItemKind::Enum(ref enum_def, ref generics) => {
                let variants = enum_from_ast(cx, enum_def);
                (Body::Enum(variants), generics)
            }
            ast::ItemKind::Struct(ref variant_data, ref generics) => {
                let (style, fields) = struct_from_ast(cx, variant_data);
                (Body::Struct(style, fields), generics)
            }
            _ => {
                cx.span_err(item.span, &format!(
                    "`#[derive({})]` may only be applied to structs and enums",
                    derive_trait));
                return Err(Error);
            }
        };

        Ok(Item {
            ident: item.ident,
            span: item.span,
            attrs: attrs,
            body: body,
            generics: generics,
        })
    }
}

fn enum_from_ast<'a>(
    cx: &ExtCtxt,
    enum_def: &'a ast::EnumDef,
) -> Vec<Variant<'a>> {
    enum_def.variants.iter()
        .map(|variant| {
            let (style, fields) = struct_from_ast(cx, &variant.node.data);
            Variant {
                ident: variant.node.name,
                attrs: attr::VariantAttrs::from_variant(cx, variant),
                style: style,
                fields: fields,
            }
        })
        .collect()
}

fn struct_from_ast<'a>(
    cx: &ExtCtxt,
    variant_data: &'a ast::VariantData,
) -> (Style, Vec<Field<'a>>) {
    match *variant_data {
        ast::VariantData::Struct(ref fields, _) => {
            (Style::Struct, fields_from_ast(cx, fields))
        }
        ast::VariantData::Tuple(ref fields, _) if fields.len() == 1 => {
            (Style::Newtype, fields_from_ast(cx, fields))
        }
        ast::VariantData::Tuple(ref fields, _) => {
            (Style::Tuple, fields_from_ast(cx, fields))
        }
        ast::VariantData::Unit(_) => {
            (Style::Unit, Vec::new())
        }
    }
}

fn fields_from_ast<'a>(
    cx: &ExtCtxt,
    fields: &'a [ast::StructField],
) -> Vec<Field<'a>> {
    fields.iter()
        .enumerate()
        .map(|(i, field)| {
            Field {
                ident: field.ident,
                span: field.span,
                attrs: attr::FieldAttrs::from_field(cx, i, field),
                ty: &field.ty,
            }
        })
        .collect()
}

impl<'a> Body<'a> {
    pub fn all_fields(&'a self) -> Box<Iterator<Item=&'a Field<'a>> + 'a> {
        match *self {
            Body::Enum(ref variants) => {
                Box::new(variants.iter()
                             .flat_map(|variant| variant.fields.iter()))
            }
            Body::Struct(_, ref fields) => {
                Box::new(fields.iter())
            }
        }
    }
}
