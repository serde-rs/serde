use syntax::ast;
use syntax::attr;
use syntax::ext::base::ExtCtxt;
use syntax::print::pprust::meta_item_to_string;
use syntax::ptr::P;

use aster::AstBuilder;

use error::Error;

/// Represents container (e.g. struct) attribute information
#[derive(Debug)]
pub struct ContainerAttrs {
    deny_unknown_fields: bool,
}

impl ContainerAttrs {
    /// Extract out the `#[serde(...)]` attributes from an item.
    pub fn from_item(cx: &ExtCtxt, item: &ast::Item) -> Result<ContainerAttrs, Error> {
        let mut container_attrs = ContainerAttrs {
            deny_unknown_fields: false,
        };

        for meta_items in item.attrs().iter().filter_map(get_serde_meta_items) {
            for meta_item in meta_items {
                match meta_item.node {
                    // Parse `#[serde(deny_unknown_fields)]`
                    ast::MetaWord(ref name) if name == &"deny_unknown_fields" => {
                        container_attrs.deny_unknown_fields = true;
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

        Ok(container_attrs)
    }

    pub fn deny_unknown_fields(&self) -> bool {
        self.deny_unknown_fields
    }
}

/// Represents field attribute information
#[derive(Debug)]
pub struct FieldAttrs {
    ident: ast::Ident,
    name: Option<ast::Lit>,
    skip_serializing_field: bool,
    skip_serializing_field_if_empty: bool,
    skip_serializing_field_if_none: bool,
    use_default: bool,
}

impl FieldAttrs {
    /// Extract out the `#[serde(...)]` attributes from a struct field.
    pub fn from_field(cx: &ExtCtxt, field: &ast::StructField) -> Result<Self, Error> {
        let field_ident = match field.node.ident() {
            Some(ident) => ident,
            None => { cx.span_bug(field.span, "struct field has no name?") }
        };

        let mut skip_serializing_field = false;
        let mut skip_serializing_field_if_empty = false;
        let mut skip_serializing_field_if_none = false;
        let mut field_name = None;
        let mut use_default = false;

        for meta_items in field.node.attrs.iter().filter_map(get_serde_meta_items) {
            for meta_item in meta_items {
                match meta_item.node {
                    // Parse `#[serde(rename="foo")]`
                    ast::MetaNameValue(ref name, ref lit) if name == &"rename" => {
                        field_name = Some(lit.clone());
                    }

                    // Parse `#[serde(default)]`
                    ast::MetaWord(ref name) if name == &"default" => {
                        use_default = true;
                    }

                    // Parse `#[serde(skip_serializing)]`
                    ast::MetaWord(ref name) if name == &"skip_serializing" => {
                        skip_serializing_field = true;
                    }

                    // Parse `#[serde(skip_serializing_if_none)]`
                    ast::MetaWord(ref name) if name == &"skip_serializing_if_none" => {
                        skip_serializing_field_if_none = true;
                    }

                    // Parse `#[serde(skip_serializing_if_empty)]`
                    ast::MetaWord(ref name) if name == &"skip_serializing_if_empty" => {
                        skip_serializing_field_if_empty = true;
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

        Ok(FieldAttrs {
            ident: field_ident,
            name: field_name,
            skip_serializing_field: skip_serializing_field,
            skip_serializing_field_if_empty: skip_serializing_field_if_empty,
            skip_serializing_field_if_none: skip_serializing_field_if_none,
            use_default: use_default,
        })
    }

    pub fn from_variant(variant: &ast::Variant) -> Self {
        FieldAttrs {
            ident: variant.node.name,
            name: None,
            skip_serializing_field: false,
            skip_serializing_field_if_empty: false,
            skip_serializing_field_if_none: false,
            use_default: false,
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
        self.use_default
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

/// Extract out the `#[serde(...)]` attributes from a struct field.
pub fn get_struct_field_attrs(cx: &ExtCtxt,
                              fields: &[ast::StructField]) -> Result<Vec<FieldAttrs>, Error> {
    fields.iter()
        .map(|field| FieldAttrs::from_field(cx, field))
        .collect()
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
