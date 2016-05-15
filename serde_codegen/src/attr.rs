use std::rc::Rc;
use syntax::ast::{self, TokenTree};
use syntax::attr;
use syntax::codemap::Span;
use syntax::ext::base::ExtCtxt;
use syntax::fold::Folder;
use syntax::parse::parser::PathStyle;
use syntax::parse::token::{self, InternedString};
use syntax::parse;
use syntax::print::pprust::{lit_to_string, meta_item_to_string};
use syntax::ptr::P;

use aster::AstBuilder;

use error::Error;

#[derive(Debug)]
pub struct Name {
    ident: ast::Ident,
    serialize_name: Option<InternedString>,
    deserialize_name: Option<InternedString>,
}

impl Name {
    fn new(ident: ast::Ident) -> Self {
        Name {
            ident: ident,
            serialize_name: None,
            deserialize_name: None,
        }
    }

    /// Return the container name for the container when serializing.
    pub fn serialize_name(&self) -> InternedString {
        match self.serialize_name {
            Some(ref name) => name.clone(),
            None => self.ident.name.as_str(),
        }
    }

    /// Return the container name expression for the container when deserializing.
    pub fn serialize_name_expr(&self) -> P<ast::Expr> {
        AstBuilder::new().expr().str(self.serialize_name())
    }

    /// Return the container name for the container when deserializing.
    pub fn deserialize_name(&self) -> InternedString {
        match self.deserialize_name {
            Some(ref name) => name.clone(),
            None => self.ident.name.as_str(),
        }
    }

    /// Return the container name expression for the container when deserializing.
    pub fn deserialize_name_expr(&self) -> P<ast::Expr> {
        AstBuilder::new().expr().str(self.deserialize_name())
    }
}

/// Represents container (e.g. struct) attribute information
#[derive(Debug)]
pub struct ContainerAttrs {
    name: Name,
    deny_unknown_fields: bool,
}

impl ContainerAttrs {
    /// Extract out the `#[serde(...)]` attributes from an item.
    pub fn from_item(cx: &ExtCtxt, item: &ast::Item) -> Result<Self, Error> {
        let mut container_attrs = ContainerAttrs {
            name: Name::new(item.ident),
            deny_unknown_fields: false,
        };

        for meta_items in item.attrs().iter().filter_map(get_serde_meta_items) {
            for meta_item in meta_items {
                match meta_item.node {
                    // Parse `#[serde(rename="foo")]`
                    ast::MetaItemKind::NameValue(ref name, ref lit) if name == &"rename" => {
                        let s = try!(get_str_from_lit(cx, name, lit));

                        container_attrs.name.serialize_name = Some(s.clone());
                        container_attrs.name.deserialize_name = Some(s);
                    }

                    // Parse `#[serde(rename(serialize="foo", deserialize="bar"))]`
                    ast::MetaItemKind::List(ref name, ref meta_items) if name == &"rename" => {
                        let (ser_name, de_name) = try!(get_renames(cx, meta_items));

                        container_attrs.name.serialize_name = ser_name;
                        container_attrs.name.deserialize_name = de_name;
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

    pub fn name(&self) -> &Name {
        &self.name
    }

    pub fn deny_unknown_fields(&self) -> bool {
        self.deny_unknown_fields
    }
}

/// Represents variant attribute information
#[derive(Debug)]
pub struct VariantAttrs {
    name: Name,
}

impl VariantAttrs {
    pub fn from_variant(cx: &ExtCtxt, variant: &ast::Variant) -> Result<Self, Error> {
        let mut variant_attrs = VariantAttrs {
            name: Name::new(variant.node.name),
        };

        for meta_items in variant.node.attrs.iter().filter_map(get_serde_meta_items) {
            for meta_item in meta_items {
                match meta_item.node {
                    // Parse `#[serde(rename="foo")]`
                    ast::MetaItemKind::NameValue(ref name, ref lit) if name == &"rename" => {
                        let s = try!(get_str_from_lit(cx, name, lit));

                        variant_attrs.name.serialize_name = Some(s.clone());
                        variant_attrs.name.deserialize_name = Some(s);
                    }

                    // Parse `#[serde(rename(serialize="foo", deserialize="bar"))]`
                    ast::MetaItemKind::List(ref name, ref meta_items) if name == &"rename" => {
                        let (ser_name, de_name) = try!(get_renames(cx, meta_items));

                        variant_attrs.name.serialize_name = ser_name;
                        variant_attrs.name.deserialize_name = de_name;
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

    pub fn name(&self) -> &Name {
        &self.name
    }
}

/// Represents field attribute information
#[derive(Debug)]
pub struct FieldAttrs {
    name: Name,
    skip_serializing_field: bool,
    skip_deserializing_field: bool,
    skip_serializing_if: Option<ast::Path>,
    default_expr_if_missing: Option<P<ast::Expr>>,
    serialize_with: Option<ast::Path>,
    deserialize_with: Option<ast::Path>,
}

impl FieldAttrs {
    /// Extract out the `#[serde(...)]` attributes from a struct field.
    pub fn from_field(cx: &ExtCtxt,
                      index: usize,
                      field: &ast::StructField) -> Result<Self, Error> {
        let builder = AstBuilder::new();

        let field_ident = match field.ident {
            Some(ident) => ident,
            None => builder.id(index.to_string()),
        };

        let mut field_attrs = FieldAttrs {
            name: Name::new(field_ident),
            skip_serializing_field: false,
            skip_deserializing_field: false,
            skip_serializing_if: None,
            default_expr_if_missing: None,
            serialize_with: None,
            deserialize_with: None,
        };

        for meta_items in field.attrs.iter().filter_map(get_serde_meta_items) {
            for meta_item in meta_items {
                match meta_item.node {
                    // Parse `#[serde(rename="foo")]`
                    ast::MetaItemKind::NameValue(ref name, ref lit) if name == &"rename" => {
                        let s = try!(get_str_from_lit(cx, name, lit));

                        field_attrs.name.serialize_name = Some(s.clone());
                        field_attrs.name.deserialize_name = Some(s);
                    }

                    // Parse `#[serde(rename(serialize="foo", deserialize="bar"))]`
                    ast::MetaItemKind::List(ref name, ref meta_items) if name == &"rename" => {
                        let (ser_name, de_name) = try!(get_renames(cx, meta_items));

                        field_attrs.name.serialize_name = ser_name;
                        field_attrs.name.deserialize_name = de_name;
                    }

                    // Parse `#[serde(default)]`
                    ast::MetaItemKind::Word(ref name) if name == &"default" => {
                        let default_expr = builder.expr().default();
                        field_attrs.default_expr_if_missing = Some(default_expr);
                    }

                    // Parse `#[serde(default="...")]`
                    ast::MetaItemKind::NameValue(ref name, ref lit) if name == &"default" => {
                        let wrapped_expr = wrap_default(
                            try!(parse_lit_into_path(cx, name, lit)),
                        );

                        field_attrs.default_expr_if_missing = Some(wrapped_expr);
                    }

                    // Parse `#[serde(skip_serializing)]`
                    ast::MetaItemKind::Word(ref name) if name == &"skip_serializing" => {
                        field_attrs.skip_serializing_field = true;
                    }

                    // Parse `#[serde(skip_deserializing)]`
                    ast::MetaItemKind::Word(ref name) if name == &"skip_deserializing" => {
                        field_attrs.skip_deserializing_field = true;

                        // Initialize field to Default::default() unless a different
                        // default is specified by `#[serde(default="...")]`
                        if field_attrs.default_expr_if_missing.is_none() {
                            let default_expr = builder.expr().default();
                            field_attrs.default_expr_if_missing = Some(default_expr);
                        }
                    }

                    // Parse `#[serde(skip_serializing_if="...")]`
                    ast::MetaItemKind::NameValue(ref name, ref lit) if name == &"skip_serializing_if" => {
                        let path = try!(parse_lit_into_path(cx, name, lit));
                        field_attrs.skip_serializing_if = Some(path);
                    }

                    // Parse `#[serde(serialize_with="...")]`
                    ast::MetaItemKind::NameValue(ref name, ref lit) if name == &"serialize_with" => {
                        let path = try!(parse_lit_into_path(cx, name, lit));
                        field_attrs.serialize_with = Some(path);
                    }

                    // Parse `#[serde(deserialize_with="...")]`
                    ast::MetaItemKind::NameValue(ref name, ref lit) if name == &"deserialize_with" => {
                        let path = try!(parse_lit_into_path(cx, name, lit));
                        field_attrs.deserialize_with = Some(path);
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

    pub fn name(&self) -> &Name {
        &self.name
    }

    pub fn skip_serializing_field(&self) -> bool {
        self.skip_serializing_field
    }

    pub fn skip_deserializing_field(&self) -> bool {
        self.skip_deserializing_field
    }

    pub fn skip_serializing_if(&self) -> Option<&ast::Path> {
        self.skip_serializing_if.as_ref()
    }

    pub fn default_expr_if_missing(&self) -> Option<&P<ast::Expr>> {
        self.default_expr_if_missing.as_ref()
    }

    pub fn serialize_with(&self) -> Option<&ast::Path> {
        self.serialize_with.as_ref()
    }

    pub fn deserialize_with(&self) -> Option<&ast::Path> {
        self.deserialize_with.as_ref()
    }
}


/// Zip together fields and `#[serde(...)]` attributes on those fields.
pub fn fields_with_attrs<'a>(
    cx: &ExtCtxt,
    fields: &'a [ast::StructField],
) -> Result<Vec<(&'a ast::StructField, FieldAttrs)>, Error> {
    fields.iter()
        .enumerate()
        .map(|(i, field)| {
            let attrs = try!(FieldAttrs::from_field(cx, i, field));
            Ok((field, attrs))
        })
        .collect()
}

fn get_renames(cx: &ExtCtxt,
               items: &[P<ast::MetaItem>],
              )-> Result<(Option<InternedString>, Option<InternedString>), Error> {
    let mut ser_name = None;
    let mut de_name = None;

    for item in items {
        match item.node {
            ast::MetaItemKind::NameValue(ref name, ref lit) if name == &"serialize" => {
                let s = try!(get_str_from_lit(cx, name, lit));
                ser_name = Some(s);
            }

            ast::MetaItemKind::NameValue(ref name, ref lit) if name == &"deserialize" => {
                let s = try!(get_str_from_lit(cx, name, lit));
                de_name = Some(s);
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

pub fn get_serde_meta_items(attr: &ast::Attribute) -> Option<&[P<ast::MetaItem>]> {
    match attr.node.value.node {
        ast::MetaItemKind::List(ref name, ref items) if name == &"serde" => {
            attr::mark_used(&attr);
            Some(items)
        }
        _ => None
    }
}

/// This syntax folder rewrites tokens to say their spans are coming from a macro context.
struct Respanner<'a, 'b: 'a> {
    cx: &'a ExtCtxt<'b>,
}

impl<'a, 'b> Folder for Respanner<'a, 'b> {
    fn fold_tt(&mut self, tt: &TokenTree) -> TokenTree {
        match *tt {
            TokenTree::Token(span, ref tok) => {
                TokenTree::Token(
                    self.new_span(span),
                    self.fold_token(tok.clone())
                )
            }
            TokenTree::Delimited(span, ref delimed) => {
                TokenTree::Delimited(
                    self.new_span(span),
                    Rc::new(ast::Delimited {
                        delim: delimed.delim,
                        open_span: delimed.open_span,
                        tts: self.fold_tts(&delimed.tts),
                        close_span: delimed.close_span,
                    })
                )
            }
            TokenTree::Sequence(span, ref seq) => {
                TokenTree::Sequence(
                    self.new_span(span),
                    Rc::new(ast::SequenceRepetition {
                        tts: self.fold_tts(&seq.tts),
                        separator: seq.separator.clone().map(|tok| self.fold_token(tok)),
                        ..**seq
                    })
                )
            }
        }
    }

    fn new_span(&mut self, span: Span) -> Span {
        Span {
            lo: span.lo,
            hi: span.hi,
            expn_id: self.cx.backtrace(),
        }
    }
}

fn get_str_from_lit(cx: &ExtCtxt, name: &str, lit: &ast::Lit) -> Result<InternedString, Error> {
    match lit.node {
        ast::LitKind::Str(ref s, _) => Ok(s.clone()),
        _ => {
            cx.span_err(
                lit.span,
                &format!("serde annotation `{}` must be a string, not `{}`",
                         name,
                         lit_to_string(lit)));

            return Err(Error);
        }
    }
}

fn parse_lit_into_path(cx: &ExtCtxt, name: &str, lit: &ast::Lit) -> Result<ast::Path, Error> {
    let source = try!(get_str_from_lit(cx, name, lit));

    // If we just parse the string into an expression, any syntax errors in the source will only
    // have spans that point inside the string, and not back to the attribute. So to have better
    // error reporting, we'll first parse the string into a token tree. Then we'll update those
    // spans to say they're coming from a macro context that originally came from the attribute,
    // and then finally parse them into an expression.
    let tts = panictry!(parse::parse_tts_from_source_str(
        format!("<serde {} expansion>", name),
        (*source).to_owned(),
        cx.cfg(),
        cx.parse_sess()));

    // Respan the spans to say they are all coming from this macro.
    let tts = Respanner { cx: cx }.fold_tts(&tts);

    let mut parser = parse::new_parser_from_tts(cx.parse_sess(), cx.cfg(), tts);

    let path = match parser.parse_path(PathStyle::Type) {
        Ok(path) => path,
        Err(mut e) => {
            e.emit();
            return Err(Error);
        }
    };

    // Make sure to error out if there are trailing characters in the stream.
    match parser.expect(&token::Eof) {
        Ok(()) => { }
        Err(mut e) => {
            e.emit();
            return Err(Error);
        }
    }

    Ok(path)
}

/// This function wraps the expression in `#[serde(default="...")]` in a function to prevent it
/// from accessing the internal `Deserialize` state.
fn wrap_default(path: ast::Path) -> P<ast::Expr> {
    AstBuilder::new().expr().call()
        .build_path(path)
        .build()
}
