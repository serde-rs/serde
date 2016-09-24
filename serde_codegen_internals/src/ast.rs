use syn;
use attr;
use Ctxt;

pub struct Item<'a> {
    pub ident: syn::Ident,
    pub attrs: attr::Item,
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

pub enum Style {
    Struct,
    Tuple,
    Newtype,
    Unit,
}

impl<'a> Item<'a> {
    pub fn from_ast(cx: &Ctxt, item: &'a syn::MacroInput) -> Item<'a> {
        let attrs = attr::Item::from_ast(cx, item);

        let body = match item.body {
            syn::Body::Enum(ref variants) => {
                Body::Enum(enum_from_ast(cx, variants))
            }
            syn::Body::Struct(ref variant_data) => {
                let (style, fields) = struct_from_ast(cx, variant_data);
                Body::Struct(style, fields)
            }
        };

        Item {
            ident: item.ident.clone(),
            attrs: attrs,
            body: body,
            generics: &item.generics,
        }
    }
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

fn enum_from_ast<'a>(cx: &Ctxt, variants: &'a [syn::Variant]) -> Vec<Variant<'a>> {
    variants.iter()
        .map(|variant| {
            let (style, fields) = struct_from_ast(cx, &variant.data);
            Variant {
                ident: variant.ident.clone(),
                attrs: attr::Variant::from_ast(cx, variant),
                style: style,
                fields: fields,
            }
        })
        .collect()
}

fn struct_from_ast<'a>(cx: &Ctxt, data: &'a syn::VariantData) -> (Style, Vec<Field<'a>>) {
    match *data {
        syn::VariantData::Struct(ref fields) => {
            (Style::Struct, fields_from_ast(cx, fields))
        }
        syn::VariantData::Tuple(ref fields) if fields.len() == 1 => {
            (Style::Newtype, fields_from_ast(cx, fields))
        }
        syn::VariantData::Tuple(ref fields) => {
            (Style::Tuple, fields_from_ast(cx, fields))
        }
        syn::VariantData::Unit => {
            (Style::Unit, Vec::new())
        }
    }
}

fn fields_from_ast<'a>(cx: &Ctxt, fields: &'a [syn::Field]) -> Vec<Field<'a>> {
    fields.iter()
        .enumerate()
        .map(|(i, field)| {
            Field {
                ident: field.ident.clone(),
                attrs: attr::Field::from_ast(cx, i, field),
                ty: &field.ty,
            }
        })
        .collect()
}
