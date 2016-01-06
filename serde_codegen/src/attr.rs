use std::collections::HashMap;
use std::collections::HashSet;

use syntax::ast;
use syntax::attr;
use syntax::ext::base::ExtCtxt;
use syntax::ptr::P;

use aster;

/// Represents field name information
#[derive(Debug)]
pub enum FieldNames {
    Global(P<ast::Expr>),
    Format{
        formats: HashMap<P<ast::Expr>, P<ast::Expr>>,
        default: P<ast::Expr>,
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
    /// Return a set of formats that the field has attributes for.
    pub fn formats(&self) -> HashSet<P<ast::Expr>> {
        match self.names {
            FieldNames::Format{ref formats, default: _} => {
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
            FieldNames::Format{formats: _, ref default} => default,
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

pub struct FieldAttrsBuilder<'a> {
    builder: &'a aster::AstBuilder,
    skip_serializing_field: bool,
    skip_serializing_field_if_empty: bool,
    skip_serializing_field_if_none: bool,
    name: Option<P<ast::Expr>>,
    format_rename: HashMap<P<ast::Expr>, P<ast::Expr>>,
    use_default: bool,
}

impl<'a> FieldAttrsBuilder<'a> {
    pub fn new(builder: &'a aster::AstBuilder) -> FieldAttrsBuilder<'a> {
        FieldAttrsBuilder {
            builder: builder,
            skip_serializing_field: false,
            skip_serializing_field_if_empty: false,
            skip_serializing_field_if_none: false,
            name: None,
            format_rename: HashMap::new(),
            use_default: false,
        }
    }

    pub fn field(mut self, field: &ast::StructField) -> FieldAttrsBuilder<'a> {
        match field.node.kind {
            ast::NamedField(name, _) => {
                self.name = Some(self.builder.expr().str(name));
            }
            ast::UnnamedField(_) => { }
        };

        self.attrs(&field.node.attrs)
    }

    pub fn attrs(self, attrs: &[ast::Attribute]) -> FieldAttrsBuilder<'a> {
        attrs.iter().fold(self, FieldAttrsBuilder::attr)
    }

    pub fn attr(self, attr: &ast::Attribute) -> FieldAttrsBuilder<'a> {
        match attr.node.value.node {
            ast::MetaList(ref name, ref items) if name == &"serde" => {
                attr::mark_used(&attr);
                items.iter().fold(self, FieldAttrsBuilder::meta_item)
            }
            _ => {
                self
            }
        }
    }

    pub fn meta_item(mut self, meta_item: &P<ast::MetaItem>) -> FieldAttrsBuilder<'a> {
        match meta_item.node {
            ast::MetaNameValue(ref name, ref lit) if name == &"rename" => {
                let expr = self.builder.expr().build_lit(P(lit.clone()));

                self.name(expr)
            }
            ast::MetaList(ref name, ref items) if name == &"rename" => {
                for item in items {
                    match item.node {
                        ast::MetaNameValue(ref name, ref lit) => {
                            let name = self.builder.expr().str(name);
                            let expr = self.builder.expr().build_lit(P(lit.clone()));

                            self = self.format_rename(name, expr);
                        }
                        _ => { }
                    }
                }
                self
            }
            ast::MetaWord(ref name) if name == &"default" => {
                self.default()
            }
            ast::MetaWord(ref name) if name == &"skip_serializing" => {
                self.skip_serializing_field()
            }
            ast::MetaWord(ref name) if name == &"skip_serializing_if_empty" => {
                self.skip_serializing_field_if_empty()
            }
            ast::MetaWord(ref name) if name == &"skip_serializing_if_none" => {
                self.skip_serializing_field_if_none()
            }
            _ => {
                // Ignore unknown meta variables for now.
                self
            }
        }
    }

    pub fn skip_serializing_field(mut self) -> FieldAttrsBuilder<'a> {
        self.skip_serializing_field = true;
        self
    }

    pub fn skip_serializing_field_if_empty(mut self) -> FieldAttrsBuilder<'a> {
        self.skip_serializing_field_if_empty = true;
        self
    }

    pub fn skip_serializing_field_if_none(mut self) -> FieldAttrsBuilder<'a> {
        self.skip_serializing_field_if_none = true;
        self
    }

    pub fn name(mut self, name: P<ast::Expr>) -> FieldAttrsBuilder<'a> {
        self.name = Some(name);
        self
    }

    pub fn format_rename(mut self, format: P<ast::Expr>, name: P<ast::Expr>) -> FieldAttrsBuilder<'a> {
        self.format_rename.insert(format, name);
        self
    }

    pub fn default(mut self) -> FieldAttrsBuilder<'a> {
        self.use_default = true;
        self
    }

    pub fn build(self) -> FieldAttrs {
        let name = self.name.expect("here");
        let names = if self.format_rename.is_empty() {
            FieldNames::Global(name)
        } else {
            FieldNames::Format {
                formats: self.format_rename,
                default: name,
            }
        };

        FieldAttrs {
            skip_serializing_field: self.skip_serializing_field,
            skip_serializing_field_if_empty: self.skip_serializing_field_if_empty,
            skip_serializing_field_if_none: self.skip_serializing_field_if_none,
            names: names,
            use_default: self.use_default,
        }
    }
}

/// Represents container (e.g. struct) attribute information
#[derive(Debug)]
pub struct ContainerAttrs {
    disallow_unknown: bool,
}

impl ContainerAttrs {
    pub fn disallow_unknown(&self) -> bool {
        self.disallow_unknown
    }
}

pub struct ContainerAttrsBuilder {
    disallow_unknown: bool,
}

impl ContainerAttrsBuilder {
    pub fn new() -> ContainerAttrsBuilder {
        ContainerAttrsBuilder {
            disallow_unknown: false,
        }
    }

    pub fn attrs(self, attrs: &[ast::Attribute]) -> ContainerAttrsBuilder {
        attrs.iter().fold(self, ContainerAttrsBuilder::attr)
    }

    pub fn attr(self, attr: &ast::Attribute) -> ContainerAttrsBuilder {
        match attr.node.value.node {
            ast::MetaList(ref name, ref items) if name == &"serde" => {
                attr::mark_used(&attr);
                items.iter().fold(self, ContainerAttrsBuilder::meta_item)
            }
            _ => {
                self
            }
        }
    }

    pub fn meta_item(self, meta_item: &P<ast::MetaItem>) -> ContainerAttrsBuilder {
        match meta_item.node {
            ast::MetaWord(ref name) if name == &"disallow_unknown" => {
                self.disallow_unknown()
            }
            _ => {
                // Ignore unknown meta variables for now.
                self
            }
        }
    }

    pub fn disallow_unknown(mut self) -> ContainerAttrsBuilder {
        self.disallow_unknown = true;
        self
    }

    pub fn build(self) -> ContainerAttrs {
        ContainerAttrs {
            disallow_unknown: self.disallow_unknown,
        }
    }
}
