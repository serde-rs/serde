use syntax::ast;
use syntax::attr;
use syntax::ext::base::ExtCtxt;
use syntax::print::pprust::{lit_to_string, meta_item_to_string};
use syntax::parse;
use syntax::ptr::P;

use aster::AstBuilder;

use error::Error;

/// Represents container (e.g. struct) attribute information
#[derive(Debug)]
pub struct ContainerAttrs {
    ident: ast::Ident,
    serialize_name: Option<ast::Lit>,
    deserialize_name: Option<ast::Lit>,
    deny_unknown_fields: bool,
}

impl ContainerAttrs {
    /// Extract out the `#[serde(...)]` attributes from an item.
    pub fn from_item(cx: &ExtCtxt, item: &ast::Item) -> Result<ContainerAttrs, Error> {
        let mut container_attrs = ContainerAttrs {
            ident: item.ident,
            serialize_name: None,
            deserialize_name: None,
            deny_unknown_fields: false,
        };

        for meta_items in item.attrs().iter().filter_map(get_serde_meta_items) {
            for meta_item in meta_items {
                match meta_item.node {
                    // Parse `#[serde(rename="foo")]`
                    ast::MetaItemKind::NameValue(ref name, ref lit) if name == &"rename" => {
                        container_attrs.serialize_name = Some(lit.clone());
                        container_attrs.deserialize_name = Some(lit.clone());
                    }

                    // Parse `#[serde(rename(serialize="foo", deserialize="bar"))]`
                    ast::MetaItemKind::List(ref name, ref meta_items) if name == &"rename" => {
                        let (ser_name, de_name) = try!(get_renames(cx, meta_items));
                        container_attrs.serialize_name = ser_name;
                        container_attrs.deserialize_name = de_name;
                    }

                    // Parse `#[serde(deny_unknown_fields)]`
                    ast::MetaItemKind::Word(ref name) if name == &"deny_unknown_fields" => {
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

    /// Return the string expression of the field ident.
    pub fn ident_expr(&self) -> P<ast::Expr> {
        AstBuilder::new().expr().str(self.ident)
    }

    /// Return the field name for the field when serializing.
    pub fn serialize_name_expr(&self) -> P<ast::Expr> {
        match self.serialize_name {
            Some(ref name) => AstBuilder::new().expr().build_lit(P(name.clone())),
            None => self.ident_expr(),
        }
    }

    /// Return the field name for the field when serializing.
    pub fn deserialize_name_expr(&self) -> P<ast::Expr> {
        match self.deserialize_name {
            Some(ref name) => AstBuilder::new().expr().build_lit(P(name.clone())),
            None => self.ident_expr(),
        }
    }

    pub fn deny_unknown_fields(&self) -> bool {
        self.deny_unknown_fields
    }
}

/// Represents variant attribute information
#[derive(Debug)]
pub struct VariantAttrs {
    ident: ast::Ident,
    serialize_name: Option<ast::Lit>,
    deserialize_name: Option<ast::Lit>,
}

impl VariantAttrs {
    pub fn from_variant(cx: &ExtCtxt, variant: &ast::Variant) -> Result<Self, Error> {
        let mut variant_attrs = VariantAttrs {
            ident: variant.node.name,
            serialize_name: None,
            deserialize_name: None,
        };

        for meta_items in variant.node.attrs.iter().filter_map(get_serde_meta_items) {
            for meta_item in meta_items {
                match meta_item.node {
                    // Parse `#[serde(rename="foo")]`
                    ast::MetaItemKind::NameValue(ref name, ref lit) if name == &"rename" => {
                        variant_attrs.serialize_name = Some(lit.clone());
                        variant_attrs.deserialize_name = Some(lit.clone());
                    }

                    // Parse `#[serde(rename(serialize="foo", deserialize="bar"))]`
                    ast::MetaItemKind::List(ref name, ref meta_items) if name == &"rename" => {
                        let (ser_name, de_name) = try!(get_renames(cx, meta_items));
                        variant_attrs.serialize_name = ser_name;
                        variant_attrs.deserialize_name = de_name;
                    }

                    _ => {
                        cx.span_err(
                            meta_item.span,
                            &format!("unknown serde variant attribute `{}`",
                                     meta_item_to_string(meta_item)));

                        return Err(Error);
                    }
                }
            }
        }

        Ok(variant_attrs)
    }

    /// Return the string expression of the field ident.
    pub fn ident_expr(&self) -> P<ast::Expr> {
        AstBuilder::new().expr().str(self.ident)
    }

    /// Return the field name for the field when serializing.
    pub fn serialize_name_expr(&self) -> P<ast::Expr> {
        match self.serialize_name {
            Some(ref name) => AstBuilder::new().expr().build_lit(P(name.clone())),
            None => self.ident_expr(),
        }
    }

    /// Return the field name for the field when serializing.
    pub fn deserialize_name_expr(&self) -> P<ast::Expr> {
        match self.deserialize_name {
            Some(ref name) => AstBuilder::new().expr().build_lit(P(name.clone())),
            None => self.ident_expr(),
        }
    }
}

/// Represents field attribute information
#[derive(Debug)]
pub struct FieldAttrs {
    ident: ast::Ident,
    serialize_name: Option<ast::Lit>,
    deserialize_name: Option<ast::Lit>,
    skip_serializing_field: bool,
    skip_serializing_field_if: Option<P<ast::Expr>>,
    skip_serializing_field_if_empty: bool,
    skip_serializing_field_if_none: bool,
    default_expr_if_missing: Option<P<ast::Expr>>,
    serialize_with: Option<P<ast::Expr>>,
}

impl FieldAttrs {
    /// Extract out the `#[serde(...)]` attributes from a struct field.
    pub fn from_field(cx: &ExtCtxt,
                      container_ty: &P<ast::Ty>,
                      generics: &ast::Generics,
                      field: &ast::StructField) -> Result<Self, Error> {
        let builder = AstBuilder::new();

        let field_ident = match field.node.ident() {
            Some(ident) => ident,
            None => { cx.span_bug(field.span, "struct field has no name?") }
        };

        let mut field_attrs = FieldAttrs {
            ident: field_ident,
            serialize_name: None,
            deserialize_name: None,
            skip_serializing_field: false,
            skip_serializing_field_if: None,
            skip_serializing_field_if_empty: false,
            skip_serializing_field_if_none: false,
            default_expr_if_missing: None,
            serialize_with: None,
        };

        for meta_items in field.node.attrs.iter().filter_map(get_serde_meta_items) {
            for meta_item in meta_items {
                match meta_item.node {
                    // Parse `#[serde(rename="foo")]`
                    ast::MetaItemKind::NameValue(ref name, ref lit) if name == &"rename" => {
                        field_attrs.serialize_name = Some(lit.clone());
                        field_attrs.deserialize_name = Some(lit.clone());
                    }

                    // Parse `#[serde(rename(serialize="foo", deserialize="bar"))]`
                    ast::MetaItemKind::List(ref name, ref meta_items) if name == &"rename" => {
                        let (ser_name, de_name) = try!(get_renames(cx, meta_items));
                        field_attrs.serialize_name = ser_name;
                        field_attrs.deserialize_name = de_name;
                    }

                    // Parse `#[serde(default)]`
                    ast::MetaItemKind::Word(ref name) if name == &"default" => {
                        let default_expr = builder.expr().default();
                        field_attrs.default_expr_if_missing = Some(default_expr);
                    }

                    // Parse `#[serde(default="...")]`
                    ast::MetaItemKind::NameValue(ref name, ref lit) if name == &"default" => {
                        let wrapped_expr = wrap_default(
                            cx,
                            &field.node.ty,
                            generics,
                            try!(parse_lit_into_expr(cx, name, lit)),
                        );

                        field_attrs.default_expr_if_missing = Some(wrapped_expr);
                    }

                    // Parse `#[serde(skip_serializing)]`
                    ast::MetaItemKind::Word(ref name) if name == &"skip_serializing" => {
                        field_attrs.skip_serializing_field = true;
                    }

                    // Parse `#[serde(skip_serializing_if="...")]`
                    ast::MetaItemKind::NameValue(ref name, ref lit) if name == &"skip_serializing_if" => {
                        let expr = wrap_skip_serializing(
                            cx,
                            container_ty,
                            generics,
                            try!(parse_lit_into_expr(cx, name, lit)),
                        );

                        field_attrs.skip_serializing_field_if = Some(expr);
                    }

                    // Parse `#[serde(skip_serializing_if_none)]`
                    ast::MetaItemKind::Word(ref name) if name == &"skip_serializing_if_none" => {
                        field_attrs.skip_serializing_field_if_none = true;
                    }

                    // Parse `#[serde(skip_serializing_if_empty)]`
                    ast::MetaItemKind::Word(ref name) if name == &"skip_serializing_if_empty" => {
                        field_attrs.skip_serializing_field_if_empty = true;
                    }

                    // Parse `#[serde(serialize_with="...")]`
                    ast::MetaItemKind::NameValue(ref name, ref lit) if name == &"serialize_with" => {
                        let expr = wrap_serialize_with(
                            cx,
                            container_ty,
                            generics,
                            try!(parse_lit_into_expr(cx, name, lit)),
                        );

                        field_attrs.serialize_with = Some(expr);
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

    /// Return the string expression of the field ident.
    pub fn ident_expr(&self) -> P<ast::Expr> {
        AstBuilder::new().expr().str(self.ident)
    }

    /// Return the field name for the field when serializing.
    pub fn serialize_name_expr(&self) -> P<ast::Expr> {
        match self.serialize_name {
            Some(ref name) => AstBuilder::new().expr().build_lit(P(name.clone())),
            None => self.ident_expr(),
        }
    }

    /// Return the field name for the field when deserializing.
    pub fn deserialize_name_expr(&self) -> P<ast::Expr> {
        match self.deserialize_name {
            Some(ref name) => AstBuilder::new().expr().build_lit(P(name.clone())),
            None => self.ident_expr(),
        }
    }

    /// Predicate for using a field's default value
    pub fn expr_is_missing(&self) -> P<ast::Expr> {
        match self.default_expr_if_missing {
            Some(ref expr) => expr.clone(),
            None => {
                let name = self.ident_expr();
                AstBuilder::new().expr()
                    .try()
                    .method_call("missing_field").id("visitor")
                        .with_arg(name)
                        .build()
            }
        }
    }

    /// Predicate for ignoring a field when serializing a value
    pub fn skip_serializing_field(&self) -> bool {
        self.skip_serializing_field
    }

    pub fn skip_serializing_field_if(&self) -> Option<&P<ast::Expr>> {
        self.skip_serializing_field_if.as_ref()
    }

    pub fn skip_serializing_field_if_empty(&self) -> bool {
        self.skip_serializing_field_if_empty
    }

    pub fn skip_serializing_field_if_none(&self) -> bool {
        self.skip_serializing_field_if_none
    }

    pub fn serialize_with(&self) -> Option<&P<ast::Expr>> {
        self.serialize_with.as_ref()
    }
}


/// Extract out the `#[serde(...)]` attributes from a struct field.
pub fn get_struct_field_attrs(cx: &ExtCtxt,
                              container_ty: &P<ast::Ty>,
                              generics: &ast::Generics,
                              fields: &[ast::StructField]) -> Result<Vec<FieldAttrs>, Error> {
    fields.iter()
        .map(|field| FieldAttrs::from_field(cx, container_ty, generics, field))
        .collect()
}

fn get_renames(cx: &ExtCtxt,
               items: &[P<ast::MetaItem>]) -> Result<(Option<ast::Lit>, Option<ast::Lit>), Error> {
    let mut ser_name = None;
    let mut de_name = None;

    for item in items {
        match item.node {
            ast::MetaItemKind::NameValue(ref name, ref lit) if name == &"serialize" => {
                ser_name = Some(lit.clone());
            }

            ast::MetaItemKind::NameValue(ref name, ref lit) if name == &"deserialize" => {
                de_name = Some(lit.clone());
            }

            _ => {
                cx.span_err(
                    item.span,
                    &format!("unknown rename attribute `{}`",
                             meta_item_to_string(item)));

                return Err(Error);
            }
        }
    }

    Ok((ser_name, de_name))
}

fn get_serde_meta_items(attr: &ast::Attribute) -> Option<&[P<ast::MetaItem>]> {
    match attr.node.value.node {
        ast::MetaItemKind::List(ref name, ref items) if name == &"serde" => {
            attr::mark_used(&attr);
            Some(items)
        }
        _ => None
    }
}

fn parse_lit_into_expr(cx: &ExtCtxt, name: &str, lit: &ast::Lit) -> Result<P<ast::Expr>, Error> {
    let s: &str = match lit.node {
        ast::LitKind::Str(ref s, ast::StrStyle::Cooked) => &s,
        _ => {
            cx.span_err(
                lit.span,
                &format!("{} literal `{}` must be a string",
                         name,
                         lit_to_string(lit)));

            return Err(Error);
        }
    };

    let expr = parse::parse_expr_from_source_str("<lit expansion>".to_string(),
                                                 s.to_owned(),
                                                 cx.cfg(),
                                                 cx.parse_sess());

    Ok(expr)
}

/// This function wraps the expression in `#[serde(default="...")]` in a function to prevent it
/// from accessing the internal `Deserialize` state.
fn wrap_default(cx: &ExtCtxt,
                field_ty: &P<ast::Ty>,
                generics: &ast::Generics,
                expr: P<ast::Expr>) -> P<ast::Expr> {
    let builder = AstBuilder::new();

    // Quasi-quoting doesn't do a great job of expanding generics into paths, so manually build it.
    let fn_path = builder.path()
        .segment("__serde_default")
            .with_generics(generics.clone())
            .build()
        .build();

    let where_clause = &generics.where_clause;

    quote_expr!(cx, {
        fn __serde_default $generics() -> $field_ty $where_clause {
            $expr
        }
        $fn_path()
    })
}

/// This function wraps the expression in `#[serde(skip_serializing_if="...")]` in a trait to
/// prevent it from accessing the internal `Serialize` state.
fn wrap_skip_serializing(cx: &ExtCtxt,
                         container_ty: &P<ast::Ty>,
                         generics: &ast::Generics,
                         expr: P<ast::Expr>) -> P<ast::Expr> {
    let where_clause = &generics.where_clause;

    quote_expr!(cx, {
        trait __SerdeShouldSkipSerializing {
            fn __serde_should_skip_serializing(&self) -> bool;
        }

        impl $generics __SerdeShouldSkipSerializing for $container_ty $where_clause {
            fn __serde_should_skip_serializing(&self) -> bool {
                $expr
            }
        }

        self.value.__serde_should_skip_serializing()
    })
}

/// This function wraps the expression in `#[serde(serialize_with="...")]` in a trait to
/// prevent it from accessing the internal `Serialize` state.
fn wrap_serialize_with(cx: &ExtCtxt,
                       container_ty: &P<ast::Ty>,
                       generics: &ast::Generics,
                       expr: P<ast::Expr>) -> P<ast::Expr> {
    let where_clause = &generics.where_clause;

    quote_expr!(cx, {
        trait __SerdeSerializeWith {
            fn __serde_serialize_with<S>(&self, serializer: &mut S) -> Result<(), S::Error>
                where S: ::serde::ser::Serializer;
        }

        impl<'a, T> __SerdeSerializeWith for &'a T
            where T: 'a + __SerdeSerializeWith,
        {
            fn __serde_serialize_with<S>(&self, serializer: &mut S) -> Result<(), S::Error>
                where S: ::serde::ser::Serializer
            {
                (**self).__serde_serialize_with(serializer)
            }
        }

        impl $generics __SerdeSerializeWith for $container_ty $where_clause {
            fn __serde_serialize_with<S>(&self, serializer: &mut S) -> Result<(), S::Error>
                where S: ::serde::ser::Serializer
            {
                $expr
            }
        }

        struct __SerdeSerializeWithStruct<'a, T: 'a> {
            value: &'a T,
        }

        impl<'a, T> ::serde::ser::Serialize for __SerdeSerializeWithStruct<'a, T>
            where T: 'a + __SerdeSerializeWith
        {
            fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
                where S: ::serde::ser::Serializer
            {
                self.value.__serde_serialize_with(serializer)
            }
        }

        __SerdeSerializeWithStruct {
            value: &self.value,
        }
    })
}
