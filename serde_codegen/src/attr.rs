use std::collections::HashMap;
use std::collections::HashSet;

use syntax::ast;
use syntax::attr;
use syntax::ext::base::ExtCtxt;
use syntax::print::pprust::meta_item_to_string;
use syntax::ptr::P;

use aster::AstBuilder;

use error::Error;

/// Represents field name information
#[derive(Debug)]
pub enum FieldNames {
    Global(P<ast::Expr>),
    Format {
        formats: HashMap<P<ast::Expr>, P<ast::Expr>>,
        default: P<ast::Expr>,
    }
}

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
    skip_serializing_field: bool,
    skip_serializing_field_if_empty: bool,
    skip_serializing_field_if_none: bool,
    names: FieldNames,
    use_default: bool,
}

impl FieldAttrs {
    /// Extract out the `#[serde(...)]` attributes from a struct field.
    pub fn from_field(cx: &ExtCtxt, field: &ast::StructField) -> Result<Self, Error> {
        let builder = AstBuilder::new();

        let field_ident = match field.node.ident() {
            Some(ident) => ident,
            None => { cx.span_bug(field.span, "struct field has no name?") }
        };

        let mut skip_serializing_field = false;
        let mut skip_serializing_field_if_empty = false;
        let mut skip_serializing_field_if_none = false;
        let mut field_name = builder.expr().str(field_ident);
        let mut format_rename = HashMap::new();
        let mut use_default = false;

        for meta_items in field.node.attrs.iter().filter_map(get_serde_meta_items) {
            for meta_item in meta_items {
                match meta_item.node {
                    // Parse `#[serde(rename="foo")]`
                    ast::MetaNameValue(ref name, ref lit) if name == &"rename" => {
                        field_name = builder.expr().build_lit(P(lit.clone()));
                    }

                    // Parse `#[serde(rename(xml="foo", token="bar"))]`
                    ast::MetaList(ref name, ref meta_items) if name == &"rename" => {
                        for meta_item in meta_items {
                            match meta_item.node {
                                ast::MetaNameValue(ref name, ref lit) => {
                                    let name = builder.expr().str(name);
                                    let expr = builder.expr().build_lit(P(lit.clone()));
                                    format_rename.insert(name, expr);
                                }
                                _ => { }
                            }
                        }
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

        let names = if format_rename.is_empty() {
            FieldNames::Global(field_name)
        } else {
            FieldNames::Format {
                formats: format_rename,
                default: field_name,
            }
        };

        Ok(FieldAttrs {
            skip_serializing_field: skip_serializing_field,
            skip_serializing_field_if_empty: skip_serializing_field_if_empty,
            skip_serializing_field_if_none: skip_serializing_field_if_none,
            names: names,
            use_default: use_default,
        })
    }

    pub fn from_variant(variant: &ast::Variant) -> Self {
        let name = AstBuilder::new().expr().str(variant.node.name);

        FieldAttrs {
            skip_serializing_field: false,
            skip_serializing_field_if_empty: false,
            skip_serializing_field_if_none: false,
            names: FieldNames::Global(name),
            use_default: false,
        }
    }

    /// Return a set of formats that the field has attributes for.
    pub fn formats(&self) -> HashSet<P<ast::Expr>> {
        match self.names {
            FieldNames::Format { ref formats, .. } => {
                let mut set = HashSet::new();
                for (fmt, _) in formats.iter() {
                    set.insert(fmt.clone());
                };
                set
            },
            _ => HashSet::new()
        }
    }

    /// Return an expression for the field key name for serialisation.
    ///
    /// The resulting expression assumes that `S` refers to a type
    /// that implements `Serializer`.
    pub fn serializer_key_expr(&self, cx: &ExtCtxt) -> P<ast::Expr> {
        match self.names {
            FieldNames::Global(ref name) => name.clone(),
            FieldNames::Format { ref formats, ref default } => {
                let arms = formats.iter()
                    .map(|(fmt, lit)| {
                        quote_arm!(cx, $fmt => { $lit })
                    })
                    .collect::<Vec<_>>();
                quote_expr!(cx,
                    match S::format() {
                        $arms
                        _ => { $default }
                    }
                )
            },
        }
    }

    /// Return the default field name for the field.
    pub fn default_key_expr(&self) -> &P<ast::Expr> {
        match self.names {
            FieldNames::Global(ref expr) => expr,
            FieldNames::Format { ref default, .. } => default,
        }
    }

    /// Return the field name for the field in the specified format.
    pub fn key_expr(&self, format: &P<ast::Expr>) -> &P<ast::Expr> {
        match self.names {
            FieldNames::Global(ref expr) => expr,
            FieldNames::Format { ref formats, ref default } => {
                formats.get(format).unwrap_or(default)
            }
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
