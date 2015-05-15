use std::collections::HashMap;
use std::collections::HashSet;

use syntax::ast;
use syntax::ext::base::ExtCtxt;
use syntax::ptr::P;

/// Represents field name information
pub enum FieldNames {
    Global(P<ast::Expr>),
    Format{
        formats: HashMap<P<ast::Expr>, P<ast::Expr>>,
        default: P<ast::Expr>,
    }
}

/// Represents field attribute information
pub struct FieldAttrs {
    names: FieldNames,
    use_default: bool,
}

impl FieldAttrs {

    /// Create a FieldAttr with a single default field name
    pub fn new(default_value: bool, name: P<ast::Expr>) -> FieldAttrs {
        FieldAttrs {
            names: FieldNames::Global(name),
            use_default: default_value,
        }
    }

    /// Create a FieldAttr with format specific field names
    pub fn new_with_formats(
        default_value: bool,
        default_name: P<ast::Expr>,
        formats: HashMap<P<ast::Expr>, P<ast::Expr>>,
        ) -> FieldAttrs {
        FieldAttrs {
            names:  FieldNames::Format {
                formats: formats,
                default: default_name,
            },
            use_default: default_value,
        }
    }

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
    pub fn serializer_key_expr(self, cx: &ExtCtxt) -> P<ast::Expr> {
        match self.names {
            FieldNames::Global(x) => x,
            FieldNames::Format{formats, default} => {
                let arms = formats.iter()
                    .map(|(fmt, lit)| {
                        quote_arm!(cx, $fmt => { $lit })
                    })
                    .collect::<Vec<_>>();
                quote_expr!(cx,
                            {
                                match S::format() {
                                    $arms
                                    _ => { $default }
                                }
                            })
            },
        }
    }

    /// Return the default field name for the field.
    pub fn default_key_expr(&self) -> &P<ast::Expr> {
        match self.names {
            FieldNames::Global(ref expr) => expr,
            FieldNames::Format{formats: _, ref default} => default
        }
    }

    /// Return the field name for the field in the specified format.
    pub fn key_expr(&self, format: &P<ast::Expr>) -> &P<ast::Expr> {
        match self.names {
            FieldNames::Global(ref expr) =>
                expr,
            FieldNames::Format{ref formats, ref default} =>
                formats.get(format).unwrap_or(default)
        }
    }

    /// Predicate for using a field's default value
    pub fn use_default(&self) -> bool {
        self.use_default
    }
}
