use syntax::ast;
use syntax::attr;
use syntax::ext::base::ExtCtxt;
use syntax::print::pprust::meta_item_to_string;
use syntax::ptr::P;

use aster::AstBuilder;

use error::Error;

/// Represents field attribute information
#[derive(Debug)]
pub struct FieldAttrs {
    ident: ast::Ident,
    name: Option<ast::Lit>,
    use_default_if_missing: bool,
    skip_serializing_field: bool,
    skip_serializing_field_if_empty: bool,
    skip_serializing_field_if_none: bool,
}

impl FieldAttrs {
    pub fn new(ident: ast::Ident) -> Self {
        FieldAttrs {
            ident: ident,
            name: None,
            use_default_if_missing: false,
            skip_serializing_field: false,
            skip_serializing_field_if_empty: false,
            skip_serializing_field_if_none: false,
        }
    }

    /// Return the default field name for the field.
    pub fn name_expr(&self) -> P<ast::Expr> {
        match self.name {
            Some(ref name) => AstBuilder::new().expr().build_lit(P(name.clone())),
            None => AstBuilder::new().expr().str(self.ident),
        }
    }

    /// Predicate for using a field's default value
    pub fn use_default(&self) -> bool {
        self.use_default_if_missing
    }

    /// Predicate for ignoring a field when serializing a value
    pub fn skip_serializing_field(&self) -> bool {
        self.skip_serializing_field
    }

    pub fn skip_serializing_field_if_empty(&self) -> bool {
        self.skip_serializing_field_if_empty
    }

    pub fn skip_serializing_field_if_none(&self) -> bool {
        self.skip_serializing_field_if_none
    }
}

/// Represents container (e.g. struct) attribute information
#[derive(Debug)]
pub struct ContainerAttrs {
    deny_unknown_fields: bool,
}

impl ContainerAttrs {
    pub fn deny_unknown_fields(&self) -> bool {
        self.deny_unknown_fields
    }
}

/// Extract out the `#[serde(...)]` attributes from an item.
pub fn get_container_attrs(cx: &ExtCtxt, item: &ast::Item) -> Result<ContainerAttrs, Error> {
    let mut deny_unknown_fields = false;

    for meta_items in item.attrs().iter().filter_map(get_serde_meta_items) {
        for meta_item in meta_items {
            match meta_item.node {
                // Parse `#[serde(deny_unknown_fields)]`
                ast::MetaWord(ref name) if name == &"deny_unknown_fields" => {
                    deny_unknown_fields = true;
                }

                _ => {
                    cx.span_err(
                        meta_item.span,
                        &format!("unknown serde container attribute `{}`",
                                 meta_item_to_string(meta_item)));

                    return Err(Error);
                }
            }
        }
    }

    Ok(ContainerAttrs {
        deny_unknown_fields: deny_unknown_fields,
    })
}

/// Extract out the `#[serde(...)]` attributes from a struct field.
pub fn get_struct_field_attrs(cx: &ExtCtxt,
                              fields: &[ast::StructField]) -> Result<Vec<FieldAttrs>, Error> {
    let mut field_attrs = Vec::with_capacity(fields.len());

    for field in fields {
        field_attrs.push(try!(get_field_attrs(cx, field)));
    }

    Ok(field_attrs)
}


/// Extract out the `#[serde(...)]` attributes from a struct field.
fn get_field_attrs(cx: &ExtCtxt, field: &ast::StructField) -> Result<FieldAttrs, Error> {
    let builder = AstBuilder::new();

    let field_ident = match field.node.ident() {
        Some(ident) => ident,
        None => { cx.span_bug(field.span, "struct field has no name?") }
    };

    let mut field_attrs = FieldAttrs::new(field_ident);

    for meta_items in field.node.attrs.iter().filter_map(get_serde_meta_items) {
        for meta_item in meta_items {
            match meta_item.node {
                // Parse `#[serde(rename="foo")]`
                ast::MetaNameValue(ref name, ref lit) if name == &"rename" => {
                    field_attrs.name = Some(lit.clone());
                }

                // Parse `#[serde(default)]`
                ast::MetaWord(ref name) if name == &"default" => {
                    field_attrs.use_default_if_missing = true;
                }

                // Parse `#[serde(skip_serializing)]`
                ast::MetaWord(ref name) if name == &"skip_serializing" => {
                    field_attrs.skip_serializing_field = true;
                }

                // Parse `#[serde(skip_serializing_if_none)]`
                ast::MetaWord(ref name) if name == &"skip_serializing_if_none" => {
                    field_attrs.skip_serializing_field_if_none = true;
                }

                // Parse `#[serde(skip_serializing_if_empty)]`
                ast::MetaWord(ref name) if name == &"skip_serializing_if_empty" => {
                    field_attrs.skip_serializing_field_if_empty = true;
                }

                _ => {
                    cx.span_err(
                        meta_item.span,
                        &format!("unknown serde field attribute `{}`",
                                 meta_item_to_string(meta_item)));

                    return Err(Error);
                }
            }
        }
    }

    Ok(field_attrs)
}

fn get_serde_meta_items(attr: &ast::Attribute) -> Option<&[P<ast::MetaItem>]> {
    match attr.node.value.node {
        ast::MetaList(ref name, ref items) if name == &"serde" => {
            attr::mark_used(&attr);
            Some(items)
        }
        _ => None
    }
}
