use std::rc::Rc;

use syntax::ast::{self, TokenTree};
use syntax::attr;
use syntax::codemap::{Span, Spanned, respan};
use syntax::ext::base::ExtCtxt;
use syntax::fold::Folder;
use syntax::parse::parser::{Parser, PathStyle};
use syntax::parse::token::{self, InternedString};
use syntax::parse;
use syntax::print::pprust::{lit_to_string, meta_item_to_string};
use syntax::ptr::P;

// This module handles parsing of `#[serde(...)]` attributes. The entrypoints
// are `attr::Item::from_ast`, `attr::Variant::from_ast`, and
// `attr::Field::from_ast`. Each returns an instance of the corresponding
// struct. Note that none of them return a Result. Unrecognized, malformed, or
// duplicated attributes result in a span_err but otherwise are ignored. The
// user will see errors simultaneously for all bad attributes in the crate
// rather than just the first.

struct Attr<'a, 'b: 'a, T> {
    cx: &'a ExtCtxt<'b>,
    name: &'static str,
    value: Option<Spanned<T>>,
}
impl<'a, 'b, T> Attr<'a, 'b, T> {
    fn none(cx: &'a ExtCtxt<'b>, name: &'static str) -> Self {
        Attr {
            cx: cx,
            name: name,
            value: None,
        }
    }

    fn set(&mut self, span: Span, t: T) {
        if let Some(Spanned { span: prev_span, .. }) = self.value {
            let mut err = self.cx.struct_span_err(
                span,
                &format!("duplicate serde attribute `{}`", self.name));
            err.span_help(prev_span, "previously set here");
            err.emit();
        } else {
            self.value = Some(respan(span, t));
        }
    }

    fn set_opt(&mut self, v: Option<Spanned<T>>) {
        if let Some(v) = v {
            self.set(v.span, v.node);
        }
    }

    fn set_if_none(&mut self, span: Span, t: T) {
        if self.value.is_none() {
            self.value = Some(respan(span, t));
        }
    }

    fn get(self) -> Option<T> {
        self.value.map(|spanned| spanned.node)
    }

    fn get_spanned(self) -> Option<Spanned<T>> {
        self.value
    }
}

struct BoolAttr<'a, 'b: 'a>(Attr<'a, 'b, ()>);
impl<'a, 'b> BoolAttr<'a, 'b> {
    fn none(cx: &'a ExtCtxt<'b>, name: &'static str) -> Self {
        BoolAttr(Attr::none(cx, name))
    }

    fn set_true(&mut self, span: Span) {
        self.0.set(span, ());
    }

    fn get(&self) -> bool {
        self.0.value.is_some()
    }
}

#[derive(Debug)]
pub struct Name {
    serialize: InternedString,
    deserialize: InternedString,
}

impl Name {
    /// Return the container name for the container when serializing.
    pub fn serialize_name(&self) -> InternedString {
        self.serialize.clone()
    }

    /// Return the container name for the container when deserializing.
    pub fn deserialize_name(&self) -> InternedString {
        self.deserialize.clone()
    }
}

/// Represents container (e.g. struct) attribute information
#[derive(Debug)]
pub struct Item {
    name: Name,
    deny_unknown_fields: bool,
    ser_bound: Option<Vec<ast::WherePredicate>>,
    de_bound: Option<Vec<ast::WherePredicate>>,
}

impl Item {
    /// Extract out the `#[serde(...)]` attributes from an item.
    pub fn from_ast(cx: &ExtCtxt, item: &ast::Item) -> Self {
        let mut ser_name = Attr::none(cx, "rename");
        let mut de_name = Attr::none(cx, "rename");
        let mut deny_unknown_fields = BoolAttr::none(cx, "deny_unknown_fields");
        let mut ser_bound = Attr::none(cx, "bound");
        let mut de_bound = Attr::none(cx, "bound");

        let ident = item.ident.name.as_str();

        for meta_items in item.attrs().iter().filter_map(get_serde_meta_items) {
            for meta_item in meta_items {
                let span = meta_item.span;
                match meta_item.node {
                    // Parse `#[serde(rename="foo")]`
                    ast::MetaItemKind::NameValue(ref name, ref lit) if name == &"rename" => {
                        if let Ok(s) = get_str_from_lit(cx, name, lit) {
                            ser_name.set(span, s.clone());
                            de_name.set(span, s);
                        }
                    }

                    // Parse `#[serde(rename(serialize="foo", deserialize="bar"))]`
                    ast::MetaItemKind::List(ref name, ref meta_items) if name == &"rename" => {
                        if let Ok((ser, de)) = get_renames(cx, meta_items) {
                            ser_name.set_opt(ser);
                            de_name.set_opt(de);
                        }
                    }

                    // Parse `#[serde(deny_unknown_fields)]`
                    ast::MetaItemKind::Word(ref name) if name == &"deny_unknown_fields" => {
                        deny_unknown_fields.set_true(span);
                    }

                    // Parse `#[serde(bound="D: Serialize")]`
                    ast::MetaItemKind::NameValue(ref name, ref lit) if name == &"bound" => {
                        if let Ok(where_predicates) = parse_lit_into_where(cx, name, lit) {
                            ser_bound.set(span, where_predicates.clone());
                            de_bound.set(span, where_predicates);
                        }
                    }

                    // Parse `#[serde(bound(serialize="D: Serialize", deserialize="D: Deserialize"))]`
                    ast::MetaItemKind::List(ref name, ref meta_items) if name == &"bound" => {
                        if let Ok((ser, de)) = get_where_predicates(cx, meta_items) {
                            ser_bound.set_opt(ser);
                            de_bound.set_opt(de);
                        }
                    }

                    _ => {
                        cx.span_err(
                            meta_item.span,
                            &format!("unknown serde container attribute `{}`",
                                     meta_item_to_string(meta_item)));
                    }
                }
            }
        }

        Item {
            name: Name {
                serialize: ser_name.get().unwrap_or(ident.clone()),
                deserialize: de_name.get().unwrap_or(ident),
            },
            deny_unknown_fields: deny_unknown_fields.get(),
            ser_bound: ser_bound.get(),
            de_bound: de_bound.get(),
        }
    }

    pub fn name(&self) -> &Name {
        &self.name
    }

    pub fn deny_unknown_fields(&self) -> bool {
        self.deny_unknown_fields
    }

    pub fn ser_bound(&self) -> Option<&[ast::WherePredicate]> {
        self.ser_bound.as_ref().map(|vec| &vec[..])
    }

    pub fn de_bound(&self) -> Option<&[ast::WherePredicate]> {
        self.de_bound.as_ref().map(|vec| &vec[..])
    }
}

/// Represents variant attribute information
#[derive(Debug)]
pub struct Variant {
    name: Name,
}

impl Variant {
    pub fn from_ast(cx: &ExtCtxt, variant: &ast::Variant) -> Self {
        let mut ser_name = Attr::none(cx, "rename");
        let mut de_name = Attr::none(cx, "rename");

        let ident = variant.node.name.name.as_str();

        for meta_items in variant.node.attrs.iter().filter_map(get_serde_meta_items) {
            for meta_item in meta_items {
                let span = meta_item.span;
                match meta_item.node {
                    // Parse `#[serde(rename="foo")]`
                    ast::MetaItemKind::NameValue(ref name, ref lit) if name == &"rename" => {
                        if let Ok(s) = get_str_from_lit(cx, name, lit) {
                            ser_name.set(span, s.clone());
                            de_name.set(span, s);
                        }
                    }

                    // Parse `#[serde(rename(serialize="foo", deserialize="bar"))]`
                    ast::MetaItemKind::List(ref name, ref meta_items) if name == &"rename" => {
                        if let Ok((ser, de)) = get_renames(cx, meta_items) {
                            ser_name.set_opt(ser);
                            de_name.set_opt(de);
                        }
                    }

                    _ => {
                        cx.span_err(
                            meta_item.span,
                            &format!("unknown serde variant attribute `{}`",
                                     meta_item_to_string(meta_item)));
                    }
                }
            }
        }

        Variant {
            name: Name {
                serialize: ser_name.get().unwrap_or(ident.clone()),
                deserialize: de_name.get().unwrap_or(ident),
            },
        }
    }

    pub fn name(&self) -> &Name {
        &self.name
    }
}

/// Represents field attribute information
#[derive(Debug)]
pub struct Field {
    name: Name,
    skip_serializing: bool,
    skip_deserializing: bool,
    skip_serializing_if: Option<ast::Path>,
    default: FieldDefault,
    serialize_with: Option<ast::Path>,
    deserialize_with: Option<ast::Path>,
    ser_bound: Option<Vec<ast::WherePredicate>>,
    de_bound: Option<Vec<ast::WherePredicate>>,
}

/// Represents the default to use for a field when deserializing.
#[derive(Debug, PartialEq)]
pub enum FieldDefault {
    /// Field must always be specified because it does not have a default.
    None,
    /// The default is given by `std::default::Default::default()`.
    Default,
    /// The default is given by this function.
    Path(ast::Path),
}

impl Field {
    /// Extract out the `#[serde(...)]` attributes from a struct field.
    pub fn from_ast(cx: &ExtCtxt,
                    index: usize,
                    field: &ast::StructField) -> Self {
        let mut ser_name = Attr::none(cx, "rename");
        let mut de_name = Attr::none(cx, "rename");
        let mut skip_serializing = BoolAttr::none(cx, "skip_serializing");
        let mut skip_deserializing = BoolAttr::none(cx, "skip_deserializing");
        let mut skip_serializing_if = Attr::none(cx, "skip_serializing_if");
        let mut default = Attr::none(cx, "default");
        let mut serialize_with = Attr::none(cx, "serialize_with");
        let mut deserialize_with = Attr::none(cx, "deserialize_with");
        let mut ser_bound = Attr::none(cx, "bound");
        let mut de_bound = Attr::none(cx, "bound");

        let ident = match field.ident {
            Some(ident) => ident.name.as_str(),
            None => token::intern_and_get_ident(&index.to_string()),
        };

        for meta_items in field.attrs.iter().filter_map(get_serde_meta_items) {
            for meta_item in meta_items {
                let span = meta_item.span;
                match meta_item.node {
                    // Parse `#[serde(rename="foo")]`
                    ast::MetaItemKind::NameValue(ref name, ref lit) if name == &"rename" => {
                        if let Ok(s) = get_str_from_lit(cx, name, lit) {
                            ser_name.set(span, s.clone());
                            de_name.set(span, s);
                        }
                    }

                    // Parse `#[serde(rename(serialize="foo", deserialize="bar"))]`
                    ast::MetaItemKind::List(ref name, ref meta_items) if name == &"rename" => {
                        if let Ok((ser, de)) = get_renames(cx, meta_items) {
                            ser_name.set_opt(ser);
                            de_name.set_opt(de);
                        }
                    }

                    // Parse `#[serde(default)]`
                    ast::MetaItemKind::Word(ref name) if name == &"default" => {
                        default.set(span, FieldDefault::Default);
                    }

                    // Parse `#[serde(default="...")]`
                    ast::MetaItemKind::NameValue(ref name, ref lit) if name == &"default" => {
                        if let Ok(path) = parse_lit_into_path(cx, name, lit) {
                            default.set(span, FieldDefault::Path(path));
                        }
                    }

                    // Parse `#[serde(skip_serializing)]`
                    ast::MetaItemKind::Word(ref name) if name == &"skip_serializing" => {
                        skip_serializing.set_true(span);
                    }

                    // Parse `#[serde(skip_deserializing)]`
                    ast::MetaItemKind::Word(ref name) if name == &"skip_deserializing" => {
                        skip_deserializing.set_true(span);
                    }

                    // Parse `#[serde(skip_serializing_if="...")]`
                    ast::MetaItemKind::NameValue(ref name, ref lit) if name == &"skip_serializing_if" => {
                        if let Ok(path) = parse_lit_into_path(cx, name, lit) {
                            skip_serializing_if.set(span, path);
                        }
                    }

                    // Parse `#[serde(serialize_with="...")]`
                    ast::MetaItemKind::NameValue(ref name, ref lit) if name == &"serialize_with" => {
                        if let Ok(path) = parse_lit_into_path(cx, name, lit) {
                            serialize_with.set(span, path);
                        }
                    }

                    // Parse `#[serde(deserialize_with="...")]`
                    ast::MetaItemKind::NameValue(ref name, ref lit) if name == &"deserialize_with" => {
                        if let Ok(path) = parse_lit_into_path(cx, name, lit) {
                            deserialize_with.set(span, path);
                        }
                    }

                    // Parse `#[serde(bound="D: Serialize")]`
                    ast::MetaItemKind::NameValue(ref name, ref lit) if name == &"bound" => {
                        if let Ok(where_predicates) = parse_lit_into_where(cx, name, lit) {
                            ser_bound.set(span, where_predicates.clone());
                            de_bound.set(span, where_predicates);
                        }
                    }

                    // Parse `#[serde(bound(serialize="D: Serialize", deserialize="D: Deserialize"))]`
                    ast::MetaItemKind::List(ref name, ref meta_items) if name == &"bound" => {
                        if let Ok((ser, de)) = get_where_predicates(cx, meta_items) {
                            ser_bound.set_opt(ser);
                            de_bound.set_opt(de);
                        }
                    }

                    _ => {
                        cx.span_err(
                            meta_item.span,
                            &format!("unknown serde field attribute `{}`",
                                     meta_item_to_string(meta_item)));
                    }
                }
            }
        }

        // Is skip_deserializing, initialize the field to Default::default()
        // unless a different default is specified by `#[serde(default="...")]`
        if let Some(Spanned { span, .. }) = skip_deserializing.0.value {
            default.set_if_none(span, FieldDefault::Default);
        }

        Field {
            name: Name {
                serialize: ser_name.get().unwrap_or(ident.clone()),
                deserialize: de_name.get().unwrap_or(ident),
            },
            skip_serializing: skip_serializing.get(),
            skip_deserializing: skip_deserializing.get(),
            skip_serializing_if: skip_serializing_if.get(),
            default: default.get().unwrap_or(FieldDefault::None),
            serialize_with: serialize_with.get(),
            deserialize_with: deserialize_with.get(),
            ser_bound: ser_bound.get(),
            de_bound: de_bound.get(),
        }
    }

    pub fn name(&self) -> &Name {
        &self.name
    }

    pub fn skip_serializing(&self) -> bool {
        self.skip_serializing
    }

    pub fn skip_deserializing(&self) -> bool {
        self.skip_deserializing
    }

    pub fn skip_serializing_if(&self) -> Option<&ast::Path> {
        self.skip_serializing_if.as_ref()
    }

    pub fn default(&self) -> &FieldDefault {
        &self.default
    }

    pub fn serialize_with(&self) -> Option<&ast::Path> {
        self.serialize_with.as_ref()
    }

    pub fn deserialize_with(&self) -> Option<&ast::Path> {
        self.deserialize_with.as_ref()
    }

    pub fn ser_bound(&self) -> Option<&[ast::WherePredicate]> {
        self.ser_bound.as_ref().map(|vec| &vec[..])
    }

    pub fn de_bound(&self) -> Option<&[ast::WherePredicate]> {
        self.de_bound.as_ref().map(|vec| &vec[..])
    }
}

fn get_ser_and_de<T, F>(
    cx: &ExtCtxt,
    attribute: &'static str,
    items: &[P<ast::MetaItem>],
    f: F
) -> Result<(Option<Spanned<T>>, Option<Spanned<T>>), ()>
    where F: Fn(&ExtCtxt, &str, &ast::Lit) -> Result<T, ()>,
{
    let mut ser_item = Attr::none(cx, attribute);
    let mut de_item = Attr::none(cx, attribute);

    for item in items {
        match item.node {
            ast::MetaItemKind::NameValue(ref name, ref lit) if name == &"serialize" => {
                if let Ok(v) = f(cx, name, lit) {
                    ser_item.set(item.span, v);
                }
            }

            ast::MetaItemKind::NameValue(ref name, ref lit) if name == &"deserialize" => {
                if let Ok(v) = f(cx, name, lit) {
                    de_item.set(item.span, v);
                }
            }

            _ => {
                cx.span_err(
                    item.span,
                    &format!("unknown {} attribute `{}`",
                             attribute,
                             meta_item_to_string(item)));

                return Err(());
            }
        }
    }

    Ok((ser_item.get_spanned(), de_item.get_spanned()))
}

fn get_renames(
    cx: &ExtCtxt,
    items: &[P<ast::MetaItem>],
) -> Result<(Option<Spanned<InternedString>>, Option<Spanned<InternedString>>), ()> {
    get_ser_and_de(cx, "rename", items, get_str_from_lit)
}

fn get_where_predicates(
    cx: &ExtCtxt,
    items: &[P<ast::MetaItem>],
) -> Result<(Option<Spanned<Vec<ast::WherePredicate>>>, Option<Spanned<Vec<ast::WherePredicate>>>), ()> {
    get_ser_and_de(cx, "bound", items, parse_lit_into_where)
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

fn get_str_from_lit(cx: &ExtCtxt, name: &str, lit: &ast::Lit) -> Result<InternedString, ()> {
    match lit.node {
        ast::LitKind::Str(ref s, _) => Ok(s.clone()),
        _ => {
            cx.span_err(
                lit.span,
                &format!("serde annotation `{}` must be a string, not `{}`",
                         name,
                         lit_to_string(lit)));

            return Err(());
        }
    }
}

// If we just parse a string literal from an attibute, any syntax errors in the
// source will only have spans that point inside the string and not back to the
// attribute. So to have better error reporting, we'll first parse the string
// into a token tree. Then we'll update those spans to say they're coming from a
// macro context that originally came from the attribnute, and then finally
// parse them into an expression or where-clause.
fn parse_string_via_tts<T, F>(cx: &ExtCtxt, name: &str, string: String, action: F) -> Result<T, ()>
    where F: for<'a> Fn(&'a mut Parser) -> parse::PResult<'a, T>,
{
    let tts = panictry!(parse::parse_tts_from_source_str(
        format!("<serde {} expansion>", name),
        string,
        cx.cfg(),
        cx.parse_sess()));

    // Respan the spans to say they are all coming from this macro.
    let tts = Respanner { cx: cx }.fold_tts(&tts);

    let mut parser = parse::new_parser_from_tts(cx.parse_sess(), cx.cfg(), tts);

    let path = match action(&mut parser) {
        Ok(path) => path,
        Err(mut e) => {
            e.emit();
            return Err(());
        }
    };

    // Make sure to error out if there are trailing characters in the stream.
    match parser.expect(&token::Eof) {
        Ok(()) => { }
        Err(mut e) => {
            e.emit();
            return Err(());
        }
    }

    Ok(path)
}

fn parse_lit_into_path(cx: &ExtCtxt, name: &str, lit: &ast::Lit) -> Result<ast::Path, ()> {
    let string = try!(get_str_from_lit(cx, name, lit)).to_string();

    parse_string_via_tts(cx, name, string, |parser| {
        parser.parse_path(PathStyle::Type)
    })
}

fn parse_lit_into_where(cx: &ExtCtxt, name: &str, lit: &ast::Lit) -> Result<Vec<ast::WherePredicate>, ()> {
    let string = try!(get_str_from_lit(cx, name, lit));
    if string.is_empty() {
        return Ok(Vec::new());
    }

    let where_string = format!("where {}", string);

    parse_string_via_tts(cx, name, where_string, |parser| {
        let where_clause = try!(parser.parse_where_clause());
        Ok(where_clause.predicates)
    })
}
