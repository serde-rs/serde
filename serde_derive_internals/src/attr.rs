// Copyright 2017 Serde Developers
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use internals::Ctxt;
use proc_macro2::{Group, Span, TokenStream, TokenTree};
use std::collections::BTreeSet;
use std::str::FromStr;
use syn;
use syn::punctuated::Punctuated;
use syn::synom::{ParseError, Synom};
use syn::Ident;
use syn::Meta::{List, NameValue, Word};
use syn::NestedMeta::{Literal, Meta};

// This module handles parsing of `#[serde(...)]` attributes. The entrypoints
// are `attr::Container::from_ast`, `attr::Variant::from_ast`, and
// `attr::Field::from_ast`. Each returns an instance of the corresponding
// struct. Note that none of them return a Result. Unrecognized, malformed, or
// duplicated attributes result in a span_err but otherwise are ignored. The
// user will see errors simultaneously for all bad attributes in the crate
// rather than just the first.

pub use internals::case::RenameRule;

#[derive(Copy, Clone)]
struct Attr<'c, T> {
    cx: &'c Ctxt,
    name: &'static str,
    value: Option<T>,
}

impl<'c, T> Attr<'c, T> {
    fn none(cx: &'c Ctxt, name: &'static str) -> Self {
        Attr {
            cx: cx,
            name: name,
            value: None,
        }
    }

    fn set(&mut self, value: T) {
        if self.value.is_some() {
            self.cx
                .error(format!("duplicate serde attribute `{}`", self.name));
        } else {
            self.value = Some(value);
        }
    }

    fn set_opt(&mut self, value: Option<T>) {
        if let Some(value) = value {
            self.set(value);
        }
    }

    fn set_if_none(&mut self, value: T) {
        if self.value.is_none() {
            self.value = Some(value);
        }
    }

    fn get(self) -> Option<T> {
        self.value
    }
}

struct BoolAttr<'c>(Attr<'c, ()>);

impl<'c> BoolAttr<'c> {
    fn none(cx: &'c Ctxt, name: &'static str) -> Self {
        BoolAttr(Attr::none(cx, name))
    }

    fn set_true(&mut self) {
        self.0.set(());
    }

    fn get(&self) -> bool {
        self.0.value.is_some()
    }
}

pub struct Name {
    serialize: String,
    deserialize: String,
}

impl Name {
    /// Return the container name for the container when serializing.
    pub fn serialize_name(&self) -> String {
        self.serialize.clone()
    }

    /// Return the container name for the container when deserializing.
    pub fn deserialize_name(&self) -> String {
        self.deserialize.clone()
    }
}

/// Represents container (e.g. struct) attribute information
pub struct Container {
    name: Name,
    deny_unknown_fields: bool,
    default: Default,
    rename_all: RenameRule,
    ser_bound: Option<Vec<syn::WherePredicate>>,
    de_bound: Option<Vec<syn::WherePredicate>>,
    tag: EnumTag,
    type_from: Option<syn::Type>,
    type_into: Option<syn::Type>,
    remote: Option<syn::Path>,
    identifier: Identifier,
    has_flatten: bool,
}

/// Styles of representing an enum.
pub enum EnumTag {
    /// The default.
    ///
    /// ```json
    /// {"variant1": {"key1": "value1", "key2": "value2"}}
    /// ```
    External,

    /// `#[serde(tag = "type")]`
    ///
    /// ```json
    /// {"type": "variant1", "key1": "value1", "key2": "value2"}
    /// ```
    Internal { tag: String },

    /// `#[serde(tag = "t", content = "c")]`
    ///
    /// ```json
    /// {"t": "variant1", "c": {"key1": "value1", "key2": "value2"}}
    /// ```
    Adjacent { tag: String, content: String },

    /// `#[serde(untagged)]`
    ///
    /// ```json
    /// {"key1": "value1", "key2": "value2"}
    /// ```
    None,
}

/// Whether this enum represents the fields of a struct or the variants of an
/// enum.
#[derive(Copy, Clone)]
pub enum Identifier {
    /// It does not.
    No,

    /// This enum represents the fields of a struct. All of the variants must be
    /// unit variants, except possibly one which is annotated with
    /// `#[serde(other)]` and is a newtype variant.
    Field,

    /// This enum represents the variants of an enum. All of the variants must
    /// be unit variants.
    Variant,
}

impl Identifier {
    #[cfg(feature = "deserialize_in_place")]
    pub fn is_some(self) -> bool {
        match self {
            Identifier::No => false,
            Identifier::Field | Identifier::Variant => true,
        }
    }
}

impl Container {
    /// Extract out the `#[serde(...)]` attributes from an item.
    pub fn from_ast(cx: &Ctxt, item: &syn::DeriveInput) -> Self {
        let mut ser_name = Attr::none(cx, "rename");
        let mut de_name = Attr::none(cx, "rename");
        let mut deny_unknown_fields = BoolAttr::none(cx, "deny_unknown_fields");
        let mut default = Attr::none(cx, "default");
        let mut rename_all = Attr::none(cx, "rename_all");
        let mut ser_bound = Attr::none(cx, "bound");
        let mut de_bound = Attr::none(cx, "bound");
        let mut untagged = BoolAttr::none(cx, "untagged");
        let mut internal_tag = Attr::none(cx, "tag");
        let mut content = Attr::none(cx, "content");
        let mut type_from = Attr::none(cx, "from");
        let mut type_into = Attr::none(cx, "into");
        let mut remote = Attr::none(cx, "remote");
        let mut field_identifier = BoolAttr::none(cx, "field_identifier");
        let mut variant_identifier = BoolAttr::none(cx, "variant_identifier");

        for meta_items in item.attrs.iter().filter_map(get_serde_meta_items) {
            for meta_item in meta_items {
                match meta_item {
                    // Parse `#[serde(rename = "foo")]`
                    Meta(NameValue(ref m)) if m.ident == "rename" => {
                        if let Ok(s) = get_lit_str(cx, m.ident.as_ref(), m.ident.as_ref(), &m.lit) {
                            ser_name.set(s.value());
                            de_name.set(s.value());
                        }
                    }

                    // Parse `#[serde(rename(serialize = "foo", deserialize = "bar"))]`
                    Meta(List(ref m)) if m.ident == "rename" => {
                        if let Ok((ser, de)) = get_renames(cx, &m.nested) {
                            ser_name.set_opt(ser.map(syn::LitStr::value));
                            de_name.set_opt(de.map(syn::LitStr::value));
                        }
                    }

                    // Parse `#[serde(rename_all = "foo")]`
                    Meta(NameValue(ref m)) if m.ident == "rename_all" => {
                        if let Ok(s) = get_lit_str(cx, m.ident.as_ref(), m.ident.as_ref(), &m.lit) {
                            match RenameRule::from_str(&s.value()) {
                                Ok(rename_rule) => rename_all.set(rename_rule),
                                Err(()) => cx.error(format!(
                                    "unknown rename rule for #[serde(rename_all \
                                     = {:?})]",
                                    s.value()
                                )),
                            }
                        }
                    }

                    // Parse `#[serde(deny_unknown_fields)]`
                    Meta(Word(word)) if word == "deny_unknown_fields" => {
                        deny_unknown_fields.set_true();
                    }

                    // Parse `#[serde(default)]`
                    Meta(Word(word)) if word == "default" => match item.data {
                        syn::Data::Struct(syn::DataStruct {
                            fields: syn::Fields::Named(_),
                            ..
                        }) => {
                            default.set(Default::Default);
                        }
                        _ => cx.error(
                            "#[serde(default)] can only be used on structs \
                             with named fields",
                        ),
                    },

                    // Parse `#[serde(default = "...")]`
                    Meta(NameValue(ref m)) if m.ident == "default" => {
                        if let Ok(path) = parse_lit_into_expr_path(cx, m.ident.as_ref(), &m.lit) {
                            match item.data {
                                syn::Data::Struct(syn::DataStruct {
                                    fields: syn::Fields::Named(_),
                                    ..
                                }) => {
                                    default.set(Default::Path(path));
                                }
                                _ => cx.error(
                                    "#[serde(default = \"...\")] can only be used \
                                     on structs with named fields",
                                ),
                            }
                        }
                    }

                    // Parse `#[serde(bound = "D: Serialize")]`
                    Meta(NameValue(ref m)) if m.ident == "bound" => {
                        if let Ok(where_predicates) =
                            parse_lit_into_where(cx, m.ident.as_ref(), m.ident.as_ref(), &m.lit)
                        {
                            ser_bound.set(where_predicates.clone());
                            de_bound.set(where_predicates);
                        }
                    }

                    // Parse `#[serde(bound(serialize = "D: Serialize", deserialize = "D: Deserialize"))]`
                    Meta(List(ref m)) if m.ident == "bound" => {
                        if let Ok((ser, de)) = get_where_predicates(cx, &m.nested) {
                            ser_bound.set_opt(ser);
                            de_bound.set_opt(de);
                        }
                    }

                    // Parse `#[serde(untagged)]`
                    Meta(Word(word)) if word == "untagged" => match item.data {
                        syn::Data::Enum(_) => {
                            untagged.set_true();
                        }
                        syn::Data::Struct(_) | syn::Data::Union(_) => {
                            cx.error("#[serde(untagged)] can only be used on enums")
                        }
                    },

                    // Parse `#[serde(tag = "type")]`
                    Meta(NameValue(ref m)) if m.ident == "tag" => {
                        if let Ok(s) = get_lit_str(cx, m.ident.as_ref(), m.ident.as_ref(), &m.lit) {
                            match item.data {
                                syn::Data::Enum(_) => {
                                    internal_tag.set(s.value());
                                }
                                syn::Data::Struct(_) | syn::Data::Union(_) => {
                                    cx.error("#[serde(tag = \"...\")] can only be used on enums")
                                }
                            }
                        }
                    }

                    // Parse `#[serde(content = "c")]`
                    Meta(NameValue(ref m)) if m.ident == "content" => {
                        if let Ok(s) = get_lit_str(cx, m.ident.as_ref(), m.ident.as_ref(), &m.lit) {
                            match item.data {
                                syn::Data::Enum(_) => {
                                    content.set(s.value());
                                }
                                syn::Data::Struct(_) | syn::Data::Union(_) => cx.error(
                                    "#[serde(content = \"...\")] can only be used on \
                                     enums",
                                ),
                            }
                        }
                    }

                    // Parse `#[serde(from = "Type")]
                    Meta(NameValue(ref m)) if m.ident == "from" => {
                        if let Ok(from_ty) = parse_lit_into_ty(cx, m.ident.as_ref(), &m.lit) {
                            type_from.set_opt(Some(from_ty));
                        }
                    }

                    // Parse `#[serde(into = "Type")]
                    Meta(NameValue(ref m)) if m.ident == "into" => {
                        if let Ok(into_ty) = parse_lit_into_ty(cx, m.ident.as_ref(), &m.lit) {
                            type_into.set_opt(Some(into_ty));
                        }
                    }

                    // Parse `#[serde(remote = "...")]`
                    Meta(NameValue(ref m)) if m.ident == "remote" => {
                        if let Ok(path) = parse_lit_into_path(cx, m.ident.as_ref(), &m.lit) {
                            if is_primitive_path(&path, "Self") {
                                remote.set(item.ident.into());
                            } else {
                                remote.set(path);
                            }
                        }
                    }

                    // Parse `#[serde(field_identifier)]`
                    Meta(Word(word)) if word == "field_identifier" => {
                        field_identifier.set_true();
                    }

                    // Parse `#[serde(variant_identifier)]`
                    Meta(Word(word)) if word == "variant_identifier" => {
                        variant_identifier.set_true();
                    }

                    Meta(ref meta_item) => {
                        cx.error(format!(
                            "unknown serde container attribute `{}`",
                            meta_item.name()
                        ));
                    }

                    Literal(_) => {
                        cx.error("unexpected literal in serde container attribute");
                    }
                }
            }
        }

        Container {
            name: Name {
                serialize: ser_name.get().unwrap_or_else(|| item.ident.to_string()),
                deserialize: de_name.get().unwrap_or_else(|| item.ident.to_string()),
            },
            deny_unknown_fields: deny_unknown_fields.get(),
            default: default.get().unwrap_or(Default::None),
            rename_all: rename_all.get().unwrap_or(RenameRule::None),
            ser_bound: ser_bound.get(),
            de_bound: de_bound.get(),
            tag: decide_tag(cx, item, &untagged, internal_tag, content),
            type_from: type_from.get(),
            type_into: type_into.get(),
            remote: remote.get(),
            identifier: decide_identifier(cx, item, &field_identifier, &variant_identifier),
            has_flatten: false,
        }
    }

    pub fn name(&self) -> &Name {
        &self.name
    }

    pub fn rename_all(&self) -> &RenameRule {
        &self.rename_all
    }

    pub fn deny_unknown_fields(&self) -> bool {
        self.deny_unknown_fields
    }

    pub fn default(&self) -> &Default {
        &self.default
    }

    pub fn ser_bound(&self) -> Option<&[syn::WherePredicate]> {
        self.ser_bound.as_ref().map(|vec| &vec[..])
    }

    pub fn de_bound(&self) -> Option<&[syn::WherePredicate]> {
        self.de_bound.as_ref().map(|vec| &vec[..])
    }

    pub fn tag(&self) -> &EnumTag {
        &self.tag
    }

    pub fn type_from(&self) -> Option<&syn::Type> {
        self.type_from.as_ref()
    }

    pub fn type_into(&self) -> Option<&syn::Type> {
        self.type_into.as_ref()
    }

    pub fn remote(&self) -> Option<&syn::Path> {
        self.remote.as_ref()
    }

    pub fn identifier(&self) -> Identifier {
        self.identifier
    }

    pub fn has_flatten(&self) -> bool {
        self.has_flatten
    }

    pub fn mark_has_flatten(&mut self) {
        self.has_flatten = true;
    }
}

fn decide_tag(
    cx: &Ctxt,
    item: &syn::DeriveInput,
    untagged: &BoolAttr,
    internal_tag: Attr<String>,
    content: Attr<String>,
) -> EnumTag {
    match (untagged.get(), internal_tag.get(), content.get()) {
        (false, None, None) => EnumTag::External,
        (true, None, None) => EnumTag::None,
        (false, Some(tag), None) => {
            // Check that there are no tuple variants.
            if let syn::Data::Enum(ref data) = item.data {
                for variant in &data.variants {
                    match variant.fields {
                        syn::Fields::Named(_) | syn::Fields::Unit => {}
                        syn::Fields::Unnamed(ref fields) => {
                            if fields.unnamed.len() != 1 {
                                cx.error(
                                    "#[serde(tag = \"...\")] cannot be used with tuple \
                                     variants",
                                );
                                break;
                            }
                        }
                    }
                }
            }
            EnumTag::Internal { tag: tag }
        }
        (true, Some(_), None) => {
            cx.error("enum cannot be both untagged and internally tagged");
            EnumTag::External // doesn't matter, will error
        }
        (false, None, Some(_)) => {
            cx.error("#[serde(tag = \"...\", content = \"...\")] must be used together");
            EnumTag::External
        }
        (true, None, Some(_)) => {
            cx.error("untagged enum cannot have #[serde(content = \"...\")]");
            EnumTag::External
        }
        (false, Some(tag), Some(content)) => EnumTag::Adjacent {
            tag: tag,
            content: content,
        },
        (true, Some(_), Some(_)) => {
            cx.error("untagged enum cannot have #[serde(tag = \"...\", content = \"...\")]");
            EnumTag::External
        }
    }
}

fn decide_identifier(
    cx: &Ctxt,
    item: &syn::DeriveInput,
    field_identifier: &BoolAttr,
    variant_identifier: &BoolAttr,
) -> Identifier {
    match (&item.data, field_identifier.get(), variant_identifier.get()) {
        (_, false, false) => Identifier::No,
        (_, true, true) => {
            cx.error("`field_identifier` and `variant_identifier` cannot both be set");
            Identifier::No
        }
        (&syn::Data::Enum(_), true, false) => Identifier::Field,
        (&syn::Data::Enum(_), false, true) => Identifier::Variant,
        (&syn::Data::Struct(_), true, false) | (&syn::Data::Union(_), true, false) => {
            cx.error("`field_identifier` can only be used on an enum");
            Identifier::No
        }
        (&syn::Data::Struct(_), false, true) | (&syn::Data::Union(_), false, true) => {
            cx.error("`variant_identifier` can only be used on an enum");
            Identifier::No
        }
    }
}

/// Represents variant attribute information
pub struct Variant {
    name: Name,
    ser_renamed: bool,
    de_renamed: bool,
    rename_all: RenameRule,
    ser_bound: Option<Vec<syn::WherePredicate>>,
    de_bound: Option<Vec<syn::WherePredicate>>,
    skip_deserializing: bool,
    skip_serializing: bool,
    other: bool,
    serialize_with: Option<syn::ExprPath>,
    deserialize_with: Option<syn::ExprPath>,
    borrow: Option<syn::Meta>,
}

impl Variant {
    pub fn from_ast(cx: &Ctxt, variant: &syn::Variant) -> Self {
        let mut ser_name = Attr::none(cx, "rename");
        let mut de_name = Attr::none(cx, "rename");
        let mut skip_deserializing = BoolAttr::none(cx, "skip_deserializing");
        let mut skip_serializing = BoolAttr::none(cx, "skip_serializing");
        let mut rename_all = Attr::none(cx, "rename_all");
        let mut ser_bound = Attr::none(cx, "bound");
        let mut de_bound = Attr::none(cx, "bound");
        let mut other = BoolAttr::none(cx, "other");
        let mut serialize_with = Attr::none(cx, "serialize_with");
        let mut deserialize_with = Attr::none(cx, "deserialize_with");
        let mut borrow = Attr::none(cx, "borrow");

        for meta_items in variant.attrs.iter().filter_map(get_serde_meta_items) {
            for meta_item in meta_items {
                match meta_item {
                    // Parse `#[serde(rename = "foo")]`
                    Meta(NameValue(ref m)) if m.ident == "rename" => {
                        if let Ok(s) = get_lit_str(cx, m.ident.as_ref(), m.ident.as_ref(), &m.lit) {
                            ser_name.set(s.value());
                            de_name.set(s.value());
                        }
                    }

                    // Parse `#[serde(rename(serialize = "foo", deserialize = "bar"))]`
                    Meta(List(ref m)) if m.ident == "rename" => {
                        if let Ok((ser, de)) = get_renames(cx, &m.nested) {
                            ser_name.set_opt(ser.map(syn::LitStr::value));
                            de_name.set_opt(de.map(syn::LitStr::value));
                        }
                    }

                    // Parse `#[serde(rename_all = "foo")]`
                    Meta(NameValue(ref m)) if m.ident == "rename_all" => {
                        if let Ok(s) = get_lit_str(cx, m.ident.as_ref(), m.ident.as_ref(), &m.lit) {
                            match RenameRule::from_str(&s.value()) {
                                Ok(rename_rule) => rename_all.set(rename_rule),
                                Err(()) => cx.error(format!(
                                    "unknown rename rule for #[serde(rename_all \
                                     = {:?})]",
                                    s.value()
                                )),
                            }
                        }
                    }

                    // Parse `#[serde(skip)]`
                    Meta(Word(word)) if word == "skip" => {
                        skip_serializing.set_true();
                        skip_deserializing.set_true();
                    }

                    // Parse `#[serde(skip_deserializing)]`
                    Meta(Word(word)) if word == "skip_deserializing" => {
                        skip_deserializing.set_true();
                    }

                    // Parse `#[serde(skip_serializing)]`
                    Meta(Word(word)) if word == "skip_serializing" => {
                        skip_serializing.set_true();
                    }

                    // Parse `#[serde(other)]`
                    Meta(Word(word)) if word == "other" => {
                        other.set_true();
                    }

                    // Parse `#[serde(bound = "D: Serialize")]`
                    Meta(NameValue(ref m)) if m.ident == "bound" => {
                        if let Ok(where_predicates) =
                            parse_lit_into_where(cx, m.ident.as_ref(), m.ident.as_ref(), &m.lit)
                        {
                            ser_bound.set(where_predicates.clone());
                            de_bound.set(where_predicates);
                        }
                    }

                    // Parse `#[serde(bound(serialize = "D: Serialize", deserialize = "D: Deserialize"))]`
                    Meta(List(ref m)) if m.ident == "bound" => {
                        if let Ok((ser, de)) = get_where_predicates(cx, &m.nested) {
                            ser_bound.set_opt(ser);
                            de_bound.set_opt(de);
                        }
                    }

                    // Parse `#[serde(with = "...")]`
                    Meta(NameValue(ref m)) if m.ident == "with" => {
                        if let Ok(path) = parse_lit_into_expr_path(cx, m.ident.as_ref(), &m.lit) {
                            let mut ser_path = path.clone();
                            ser_path
                                .path
                                .segments
                                .push(Ident::new("serialize", Span::call_site()).into());
                            serialize_with.set(ser_path);
                            let mut de_path = path;
                            de_path
                                .path
                                .segments
                                .push(Ident::new("deserialize", Span::call_site()).into());
                            deserialize_with.set(de_path);
                        }
                    }

                    // Parse `#[serde(serialize_with = "...")]`
                    Meta(NameValue(ref m)) if m.ident == "serialize_with" => {
                        if let Ok(path) = parse_lit_into_expr_path(cx, m.ident.as_ref(), &m.lit) {
                            serialize_with.set(path);
                        }
                    }

                    // Parse `#[serde(deserialize_with = "...")]`
                    Meta(NameValue(ref m)) if m.ident == "deserialize_with" => {
                        if let Ok(path) = parse_lit_into_expr_path(cx, m.ident.as_ref(), &m.lit) {
                            deserialize_with.set(path);
                        }
                    }

                    // Defer `#[serde(borrow)]` and `#[serde(borrow = "'a + 'b")]`
                    Meta(ref m) if m.name() == "borrow" => match variant.fields {
                        syn::Fields::Unnamed(ref fields) if fields.unnamed.len() == 1 => {
                            borrow.set(m.clone());
                        }
                        _ => {
                            cx.error("#[serde(borrow)] may only be used on newtype variants");
                        }
                    },

                    Meta(ref meta_item) => {
                        cx.error(format!(
                            "unknown serde variant attribute `{}`",
                            meta_item.name()
                        ));
                    }

                    Literal(_) => {
                        cx.error("unexpected literal in serde variant attribute");
                    }
                }
            }
        }

        let ser_name = ser_name.get();
        let ser_renamed = ser_name.is_some();
        let de_name = de_name.get();
        let de_renamed = de_name.is_some();
        Variant {
            name: Name {
                serialize: ser_name.unwrap_or_else(|| variant.ident.to_string()),
                deserialize: de_name.unwrap_or_else(|| variant.ident.to_string()),
            },
            ser_renamed: ser_renamed,
            de_renamed: de_renamed,
            rename_all: rename_all.get().unwrap_or(RenameRule::None),
            ser_bound: ser_bound.get(),
            de_bound: de_bound.get(),
            skip_deserializing: skip_deserializing.get(),
            skip_serializing: skip_serializing.get(),
            other: other.get(),
            serialize_with: serialize_with.get(),
            deserialize_with: deserialize_with.get(),
            borrow: borrow.get(),
        }
    }

    pub fn name(&self) -> &Name {
        &self.name
    }

    pub fn rename_by_rule(&mut self, rule: &RenameRule) {
        if !self.ser_renamed {
            self.name.serialize = rule.apply_to_variant(&self.name.serialize);
        }
        if !self.de_renamed {
            self.name.deserialize = rule.apply_to_variant(&self.name.deserialize);
        }
    }

    pub fn rename_all(&self) -> &RenameRule {
        &self.rename_all
    }

    pub fn ser_bound(&self) -> Option<&[syn::WherePredicate]> {
        self.ser_bound.as_ref().map(|vec| &vec[..])
    }

    pub fn de_bound(&self) -> Option<&[syn::WherePredicate]> {
        self.de_bound.as_ref().map(|vec| &vec[..])
    }

    pub fn skip_deserializing(&self) -> bool {
        self.skip_deserializing
    }

    pub fn skip_serializing(&self) -> bool {
        self.skip_serializing
    }

    pub fn other(&self) -> bool {
        self.other
    }

    pub fn serialize_with(&self) -> Option<&syn::ExprPath> {
        self.serialize_with.as_ref()
    }

    pub fn deserialize_with(&self) -> Option<&syn::ExprPath> {
        self.deserialize_with.as_ref()
    }
}

/// Represents field attribute information
pub struct Field {
    name: Name,
    ser_renamed: bool,
    de_renamed: bool,
    skip_serializing: bool,
    skip_deserializing: bool,
    skip_serializing_if: Option<syn::ExprPath>,
    default: Default,
    serialize_with: Option<syn::ExprPath>,
    deserialize_with: Option<syn::ExprPath>,
    ser_bound: Option<Vec<syn::WherePredicate>>,
    de_bound: Option<Vec<syn::WherePredicate>>,
    borrowed_lifetimes: BTreeSet<syn::Lifetime>,
    getter: Option<syn::ExprPath>,
    flatten: bool,
}

/// Represents the default to use for a field when deserializing.
pub enum Default {
    /// Field must always be specified because it does not have a default.
    None,
    /// The default is given by `std::default::Default::default()`.
    Default,
    /// The default is given by this function.
    Path(syn::ExprPath),
}

impl Default {
    #[cfg(feature = "deserialize_in_place")]
    pub fn is_none(&self) -> bool {
        match *self {
            Default::None => true,
            Default::Default | Default::Path(_) => false,
        }
    }
}

impl Field {
    /// Extract out the `#[serde(...)]` attributes from a struct field.
    pub fn from_ast(
        cx: &Ctxt,
        index: usize,
        field: &syn::Field,
        attrs: Option<&Variant>,
        container_default: &Default,
    ) -> Self {
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
        let mut borrowed_lifetimes = Attr::none(cx, "borrow");
        let mut getter = Attr::none(cx, "getter");
        let mut flatten = BoolAttr::none(cx, "flatten");

        let ident = match field.ident {
            Some(ref ident) => ident.to_string(),
            None => index.to_string(),
        };

        let variant_borrow = attrs
            .and_then(|variant| variant.borrow.as_ref())
            .map(|borrow| vec![Meta(borrow.clone())]);

        for meta_items in field
            .attrs
            .iter()
            .filter_map(get_serde_meta_items)
            .chain(variant_borrow)
        {
            for meta_item in meta_items {
                match meta_item {
                    // Parse `#[serde(rename = "foo")]`
                    Meta(NameValue(ref m)) if m.ident == "rename" => {
                        if let Ok(s) = get_lit_str(cx, m.ident.as_ref(), m.ident.as_ref(), &m.lit) {
                            ser_name.set(s.value());
                            de_name.set(s.value());
                        }
                    }

                    // Parse `#[serde(rename(serialize = "foo", deserialize = "bar"))]`
                    Meta(List(ref m)) if m.ident == "rename" => {
                        if let Ok((ser, de)) = get_renames(cx, &m.nested) {
                            ser_name.set_opt(ser.map(syn::LitStr::value));
                            de_name.set_opt(de.map(syn::LitStr::value));
                        }
                    }

                    // Parse `#[serde(default)]`
                    Meta(Word(word)) if word == "default" => {
                        default.set(Default::Default);
                    }

                    // Parse `#[serde(default = "...")]`
                    Meta(NameValue(ref m)) if m.ident == "default" => {
                        if let Ok(path) = parse_lit_into_expr_path(cx, m.ident.as_ref(), &m.lit) {
                            default.set(Default::Path(path));
                        }
                    }

                    // Parse `#[serde(skip_serializing)]`
                    Meta(Word(word)) if word == "skip_serializing" => {
                        skip_serializing.set_true();
                    }

                    // Parse `#[serde(skip_deserializing)]`
                    Meta(Word(word)) if word == "skip_deserializing" => {
                        skip_deserializing.set_true();
                    }

                    // Parse `#[serde(skip)]`
                    Meta(Word(word)) if word == "skip" => {
                        skip_serializing.set_true();
                        skip_deserializing.set_true();
                    }

                    // Parse `#[serde(skip_serializing_if = "...")]`
                    Meta(NameValue(ref m)) if m.ident == "skip_serializing_if" => {
                        if let Ok(path) = parse_lit_into_expr_path(cx, m.ident.as_ref(), &m.lit) {
                            skip_serializing_if.set(path);
                        }
                    }

                    // Parse `#[serde(serialize_with = "...")]`
                    Meta(NameValue(ref m)) if m.ident == "serialize_with" => {
                        if let Ok(path) = parse_lit_into_expr_path(cx, m.ident.as_ref(), &m.lit) {
                            serialize_with.set(path);
                        }
                    }

                    // Parse `#[serde(deserialize_with = "...")]`
                    Meta(NameValue(ref m)) if m.ident == "deserialize_with" => {
                        if let Ok(path) = parse_lit_into_expr_path(cx, m.ident.as_ref(), &m.lit) {
                            deserialize_with.set(path);
                        }
                    }

                    // Parse `#[serde(with = "...")]`
                    Meta(NameValue(ref m)) if m.ident == "with" => {
                        if let Ok(path) = parse_lit_into_expr_path(cx, m.ident.as_ref(), &m.lit) {
                            let mut ser_path = path.clone();
                            ser_path
                                .path
                                .segments
                                .push(Ident::new("serialize", Span::call_site()).into());
                            serialize_with.set(ser_path);
                            let mut de_path = path;
                            de_path
                                .path
                                .segments
                                .push(Ident::new("deserialize", Span::call_site()).into());
                            deserialize_with.set(de_path);
                        }
                    }

                    // Parse `#[serde(bound = "D: Serialize")]`
                    Meta(NameValue(ref m)) if m.ident == "bound" => {
                        if let Ok(where_predicates) =
                            parse_lit_into_where(cx, m.ident.as_ref(), m.ident.as_ref(), &m.lit)
                        {
                            ser_bound.set(where_predicates.clone());
                            de_bound.set(where_predicates);
                        }
                    }

                    // Parse `#[serde(bound(serialize = "D: Serialize", deserialize = "D: Deserialize"))]`
                    Meta(List(ref m)) if m.ident == "bound" => {
                        if let Ok((ser, de)) = get_where_predicates(cx, &m.nested) {
                            ser_bound.set_opt(ser);
                            de_bound.set_opt(de);
                        }
                    }

                    // Parse `#[serde(borrow)]`
                    Meta(Word(word)) if word == "borrow" => {
                        if let Ok(borrowable) = borrowable_lifetimes(cx, &ident, &field.ty) {
                            borrowed_lifetimes.set(borrowable);
                        }
                    }

                    // Parse `#[serde(borrow = "'a + 'b")]`
                    Meta(NameValue(ref m)) if m.ident == "borrow" => {
                        if let Ok(lifetimes) =
                            parse_lit_into_lifetimes(cx, m.ident.as_ref(), &m.lit)
                        {
                            if let Ok(borrowable) = borrowable_lifetimes(cx, &ident, &field.ty) {
                                for lifetime in &lifetimes {
                                    if !borrowable.contains(lifetime) {
                                        cx.error(format!(
                                            "field `{}` does not have lifetime {}",
                                            ident, lifetime
                                        ));
                                    }
                                }
                                borrowed_lifetimes.set(lifetimes);
                            }
                        }
                    }

                    // Parse `#[serde(getter = "...")]`
                    Meta(NameValue(ref m)) if m.ident == "getter" => {
                        if let Ok(path) = parse_lit_into_expr_path(cx, m.ident.as_ref(), &m.lit) {
                            getter.set(path);
                        }
                    }

                    // Parse `#[serde(flatten)]`
                    Meta(Word(word)) if word == "flatten" => {
                        flatten.set_true();
                    }

                    Meta(ref meta_item) => {
                        cx.error(format!(
                            "unknown serde field attribute `{}`",
                            meta_item.name()
                        ));
                    }

                    Literal(_) => {
                        cx.error("unexpected literal in serde field attribute");
                    }
                }
            }
        }

        // Is skip_deserializing, initialize the field to Default::default() unless a different
        // default is specified by `#[serde(default = "...")]` on ourselves or our container (e.g.
        // the struct we are in).
        if let Default::None = *container_default {
            if skip_deserializing.0.value.is_some() {
                default.set_if_none(Default::Default);
            }
        }

        let mut borrowed_lifetimes = borrowed_lifetimes.get().unwrap_or_default();
        if !borrowed_lifetimes.is_empty() {
            // Cow<str> and Cow<[u8]> never borrow by default:
            //
            //     impl<'de, 'a, T: ?Sized> Deserialize<'de> for Cow<'a, T>
            //
            // A #[serde(borrow)] attribute enables borrowing that corresponds
            // roughly to these impls:
            //
            //     impl<'de: 'a, 'a> Deserialize<'de> for Cow<'a, str>
            //     impl<'de: 'a, 'a> Deserialize<'de> for Cow<'a, [u8]>
            if is_cow(&field.ty, is_str) {
                let mut path = syn::Path {
                    leading_colon: None,
                    segments: Punctuated::new(),
                };
                path.segments
                    .push(Ident::new("_serde", Span::call_site()).into());
                path.segments
                    .push(Ident::new("private", Span::call_site()).into());
                path.segments
                    .push(Ident::new("de", Span::call_site()).into());
                path.segments
                    .push(Ident::new("borrow_cow_str", Span::call_site()).into());
                let expr = syn::ExprPath {
                    attrs: Vec::new(),
                    qself: None,
                    path: path,
                };
                deserialize_with.set_if_none(expr);
            } else if is_cow(&field.ty, is_slice_u8) {
                let mut path = syn::Path {
                    leading_colon: None,
                    segments: Punctuated::new(),
                };
                path.segments
                    .push(Ident::new("_serde", Span::call_site()).into());
                path.segments
                    .push(Ident::new("private", Span::call_site()).into());
                path.segments
                    .push(Ident::new("de", Span::call_site()).into());
                path.segments
                    .push(Ident::new("borrow_cow_bytes", Span::call_site()).into());
                let expr = syn::ExprPath {
                    attrs: Vec::new(),
                    qself: None,
                    path: path,
                };
                deserialize_with.set_if_none(expr);
            }
        } else if is_implicitly_borrowed(&field.ty) {
            // Types &str and &[u8] are always implicitly borrowed. No need for
            // a #[serde(borrow)].
            collect_lifetimes(&field.ty, &mut borrowed_lifetimes);
        }

        let ser_name = ser_name.get();
        let ser_renamed = ser_name.is_some();
        let de_name = de_name.get();
        let de_renamed = de_name.is_some();
        Field {
            name: Name {
                serialize: ser_name.unwrap_or_else(|| ident.clone()),
                deserialize: de_name.unwrap_or(ident),
            },
            ser_renamed: ser_renamed,
            de_renamed: de_renamed,
            skip_serializing: skip_serializing.get(),
            skip_deserializing: skip_deserializing.get(),
            skip_serializing_if: skip_serializing_if.get(),
            default: default.get().unwrap_or(Default::None),
            serialize_with: serialize_with.get(),
            deserialize_with: deserialize_with.get(),
            ser_bound: ser_bound.get(),
            de_bound: de_bound.get(),
            borrowed_lifetimes: borrowed_lifetimes,
            getter: getter.get(),
            flatten: flatten.get(),
        }
    }

    pub fn name(&self) -> &Name {
        &self.name
    }

    pub fn rename_by_rule(&mut self, rule: &RenameRule) {
        if !self.ser_renamed {
            self.name.serialize = rule.apply_to_field(&self.name.serialize);
        }
        if !self.de_renamed {
            self.name.deserialize = rule.apply_to_field(&self.name.deserialize);
        }
    }

    pub fn skip_serializing(&self) -> bool {
        self.skip_serializing
    }

    pub fn skip_deserializing(&self) -> bool {
        self.skip_deserializing
    }

    pub fn skip_serializing_if(&self) -> Option<&syn::ExprPath> {
        self.skip_serializing_if.as_ref()
    }

    pub fn default(&self) -> &Default {
        &self.default
    }

    pub fn serialize_with(&self) -> Option<&syn::ExprPath> {
        self.serialize_with.as_ref()
    }

    pub fn deserialize_with(&self) -> Option<&syn::ExprPath> {
        self.deserialize_with.as_ref()
    }

    pub fn ser_bound(&self) -> Option<&[syn::WherePredicate]> {
        self.ser_bound.as_ref().map(|vec| &vec[..])
    }

    pub fn de_bound(&self) -> Option<&[syn::WherePredicate]> {
        self.de_bound.as_ref().map(|vec| &vec[..])
    }

    pub fn borrowed_lifetimes(&self) -> &BTreeSet<syn::Lifetime> {
        &self.borrowed_lifetimes
    }

    pub fn getter(&self) -> Option<&syn::ExprPath> {
        self.getter.as_ref()
    }

    pub fn flatten(&self) -> bool {
        self.flatten
    }
}

type SerAndDe<T> = (Option<T>, Option<T>);

fn get_ser_and_de<'a, T, F>(
    cx: &Ctxt,
    attr_name: &'static str,
    metas: &'a Punctuated<syn::NestedMeta, Token![,]>,
    f: F,
) -> Result<SerAndDe<T>, ()>
where
    T: 'a,
    F: Fn(&Ctxt, &str, &str, &'a syn::Lit) -> Result<T, ()>,
{
    let mut ser_meta = Attr::none(cx, attr_name);
    let mut de_meta = Attr::none(cx, attr_name);

    for meta in metas {
        match *meta {
            Meta(NameValue(ref meta)) if meta.ident == "serialize" => {
                if let Ok(v) = f(cx, attr_name, meta.ident.as_ref(), &meta.lit) {
                    ser_meta.set(v);
                }
            }

            Meta(NameValue(ref meta)) if meta.ident == "deserialize" => {
                if let Ok(v) = f(cx, attr_name, meta.ident.as_ref(), &meta.lit) {
                    de_meta.set(v);
                }
            }

            _ => {
                cx.error(format!(
                    "malformed {0} attribute, expected `{0}(serialize = ..., \
                     deserialize = ...)`",
                    attr_name
                ));
                return Err(());
            }
        }
    }

    Ok((ser_meta.get(), de_meta.get()))
}

fn get_renames<'a>(
    cx: &Ctxt,
    items: &'a Punctuated<syn::NestedMeta, Token![,]>,
) -> Result<SerAndDe<&'a syn::LitStr>, ()> {
    get_ser_and_de(cx, "rename", items, get_lit_str)
}

fn get_where_predicates(
    cx: &Ctxt,
    items: &Punctuated<syn::NestedMeta, Token![,]>,
) -> Result<SerAndDe<Vec<syn::WherePredicate>>, ()> {
    get_ser_and_de(cx, "bound", items, parse_lit_into_where)
}

pub fn get_serde_meta_items(attr: &syn::Attribute) -> Option<Vec<syn::NestedMeta>> {
    if attr.path.segments.len() == 1 && attr.path.segments[0].ident == "serde" {
        match attr.interpret_meta() {
            Some(List(ref meta)) => Some(meta.nested.iter().cloned().collect()),
            _ => {
                // TODO: produce an error
                None
            }
        }
    } else {
        None
    }
}

fn get_lit_str<'a>(
    cx: &Ctxt,
    attr_name: &str,
    meta_item_name: &str,
    lit: &'a syn::Lit,
) -> Result<&'a syn::LitStr, ()> {
    if let syn::Lit::Str(ref lit) = *lit {
        Ok(lit)
    } else {
        cx.error(format!(
            "expected serde {} attribute to be a string: `{} = \"...\"`",
            attr_name, meta_item_name
        ));
        Err(())
    }
}

fn parse_lit_into_path(cx: &Ctxt, attr_name: &str, lit: &syn::Lit) -> Result<syn::Path, ()> {
    let string = try!(get_lit_str(cx, attr_name, attr_name, lit));
    parse_lit_str(string)
        .map_err(|_| cx.error(format!("failed to parse path: {:?}", string.value())))
}

fn parse_lit_into_expr_path(
    cx: &Ctxt,
    attr_name: &str,
    lit: &syn::Lit,
) -> Result<syn::ExprPath, ()> {
    let string = try!(get_lit_str(cx, attr_name, attr_name, lit));
    parse_lit_str(string)
        .map_err(|_| cx.error(format!("failed to parse path: {:?}", string.value())))
}

fn parse_lit_into_where(
    cx: &Ctxt,
    attr_name: &str,
    meta_item_name: &str,
    lit: &syn::Lit,
) -> Result<Vec<syn::WherePredicate>, ()> {
    let string = try!(get_lit_str(cx, attr_name, meta_item_name, lit));
    if string.value().is_empty() {
        return Ok(Vec::new());
    }

    let where_string = syn::LitStr::new(&format!("where {}", string.value()), string.span());

    parse_lit_str::<syn::WhereClause>(&where_string)
        .map(|wh| wh.predicates.into_iter().collect())
        .map_err(|err| cx.error(err))
}

fn parse_lit_into_ty(cx: &Ctxt, attr_name: &str, lit: &syn::Lit) -> Result<syn::Type, ()> {
    let string = try!(get_lit_str(cx, attr_name, attr_name, lit));

    parse_lit_str(string).map_err(|_| {
        cx.error(format!(
            "failed to parse type: {} = {:?}",
            attr_name,
            string.value()
        ))
    })
}

// Parses a string literal like "'a + 'b + 'c" containing a nonempty list of
// lifetimes separated by `+`.
fn parse_lit_into_lifetimes(
    cx: &Ctxt,
    attr_name: &str,
    lit: &syn::Lit,
) -> Result<BTreeSet<syn::Lifetime>, ()> {
    let string = try!(get_lit_str(cx, attr_name, attr_name, lit));
    if string.value().is_empty() {
        cx.error("at least one lifetime must be borrowed");
        return Err(());
    }

    struct BorrowedLifetimes(Punctuated<syn::Lifetime, Token![+]>);

    impl Synom for BorrowedLifetimes {
        named!(parse -> Self, map!(
            call!(Punctuated::parse_separated_nonempty),
            BorrowedLifetimes
        ));
    }

    if let Ok(BorrowedLifetimes(lifetimes)) = parse_lit_str(string) {
        let mut set = BTreeSet::new();
        for lifetime in lifetimes {
            if !set.insert(lifetime) {
                cx.error(format!("duplicate borrowed lifetime `{}`", lifetime));
            }
        }
        return Ok(set);
    }

    cx.error(format!(
        "failed to parse borrowed lifetimes: {:?}",
        string.value()
    ));
    Err(())
}

fn is_implicitly_borrowed(ty: &syn::Type) -> bool {
    is_implicitly_borrowed_reference(ty) || is_option(ty, is_implicitly_borrowed_reference)
}

fn is_implicitly_borrowed_reference(ty: &syn::Type) -> bool {
    is_reference(ty, is_str) || is_reference(ty, is_slice_u8)
}

// Whether the type looks like it might be `std::borrow::Cow<T>` where elem="T".
// This can have false negatives and false positives.
//
// False negative:
//
//     use std::borrow::Cow as Pig;
//
//     #[derive(Deserialize)]
//     struct S<'a> {
//         #[serde(borrow)]
//         pig: Pig<'a, str>,
//     }
//
// False positive:
//
//     type str = [i16];
//
//     #[derive(Deserialize)]
//     struct S<'a> {
//         #[serde(borrow)]
//         cow: Cow<'a, str>,
//     }
fn is_cow(ty: &syn::Type, elem: fn(&syn::Type) -> bool) -> bool {
    let path = match *ty {
        syn::Type::Path(ref ty) => &ty.path,
        _ => {
            return false;
        }
    };
    let seg = match path.segments.last() {
        Some(seg) => seg.into_value(),
        None => {
            return false;
        }
    };
    let args = match seg.arguments {
        syn::PathArguments::AngleBracketed(ref bracketed) => &bracketed.args,
        _ => {
            return false;
        }
    };
    seg.ident == "Cow" && args.len() == 2 && match (&args[0], &args[1]) {
        (&syn::GenericArgument::Lifetime(_), &syn::GenericArgument::Type(ref arg)) => elem(arg),
        _ => false,
    }
}

fn is_option(ty: &syn::Type, elem: fn(&syn::Type) -> bool) -> bool {
    let path = match *ty {
        syn::Type::Path(ref ty) => &ty.path,
        _ => {
            return false;
        }
    };
    let seg = match path.segments.last() {
        Some(seg) => seg.into_value(),
        None => {
            return false;
        }
    };
    let args = match seg.arguments {
        syn::PathArguments::AngleBracketed(ref bracketed) => &bracketed.args,
        _ => {
            return false;
        }
    };
    seg.ident == "Option" && args.len() == 1 && match args[0] {
        syn::GenericArgument::Type(ref arg) => elem(arg),
        _ => false,
    }
}

// Whether the type looks like it might be `&T` where elem="T". This can have
// false negatives and false positives.
//
// False negative:
//
//     type Yarn = str;
//
//     #[derive(Deserialize)]
//     struct S<'a> {
//         r: &'a Yarn,
//     }
//
// False positive:
//
//     type str = [i16];
//
//     #[derive(Deserialize)]
//     struct S<'a> {
//         r: &'a str,
//     }
fn is_reference(ty: &syn::Type, elem: fn(&syn::Type) -> bool) -> bool {
    match *ty {
        syn::Type::Reference(ref ty) => ty.mutability.is_none() && elem(&ty.elem),
        _ => false,
    }
}

fn is_str(ty: &syn::Type) -> bool {
    is_primitive_type(ty, "str")
}

fn is_slice_u8(ty: &syn::Type) -> bool {
    match *ty {
        syn::Type::Slice(ref ty) => is_primitive_type(&ty.elem, "u8"),
        _ => false,
    }
}

fn is_primitive_type(ty: &syn::Type, primitive: &str) -> bool {
    match *ty {
        syn::Type::Path(ref ty) => ty.qself.is_none() && is_primitive_path(&ty.path, primitive),
        _ => false,
    }
}

fn is_primitive_path(path: &syn::Path, primitive: &str) -> bool {
    path.leading_colon.is_none() && path.segments.len() == 1 && path.segments[0].ident == primitive
        && path.segments[0].arguments.is_empty()
}

// All lifetimes that this type could borrow from a Deserializer.
//
// For example a type `S<'a, 'b>` could borrow `'a` and `'b`. On the other hand
// a type `for<'a> fn(&'a str)` could not borrow `'a` from the Deserializer.
//
// This is used when there is an explicit or implicit `#[serde(borrow)]`
// attribute on the field so there must be at least one borrowable lifetime.
fn borrowable_lifetimes(
    cx: &Ctxt,
    name: &str,
    ty: &syn::Type,
) -> Result<BTreeSet<syn::Lifetime>, ()> {
    let mut lifetimes = BTreeSet::new();
    collect_lifetimes(ty, &mut lifetimes);
    if lifetimes.is_empty() {
        cx.error(format!("field `{}` has no lifetimes to borrow", name));
        Err(())
    } else {
        Ok(lifetimes)
    }
}

fn collect_lifetimes(ty: &syn::Type, out: &mut BTreeSet<syn::Lifetime>) {
    match *ty {
        syn::Type::Slice(ref ty) => {
            collect_lifetimes(&ty.elem, out);
        }
        syn::Type::Array(ref ty) => {
            collect_lifetimes(&ty.elem, out);
        }
        syn::Type::Ptr(ref ty) => {
            collect_lifetimes(&ty.elem, out);
        }
        syn::Type::Reference(ref ty) => {
            out.extend(ty.lifetime.iter().cloned());
            collect_lifetimes(&ty.elem, out);
        }
        syn::Type::Tuple(ref ty) => for elem in &ty.elems {
            collect_lifetimes(elem, out);
        },
        syn::Type::Path(ref ty) => {
            if let Some(ref qself) = ty.qself {
                collect_lifetimes(&qself.ty, out);
            }
            for seg in &ty.path.segments {
                if let syn::PathArguments::AngleBracketed(ref bracketed) = seg.arguments {
                    for arg in &bracketed.args {
                        match *arg {
                            syn::GenericArgument::Lifetime(ref lifetime) => {
                                out.insert(lifetime.clone());
                            }
                            syn::GenericArgument::Type(ref ty) => {
                                collect_lifetimes(ty, out);
                            }
                            syn::GenericArgument::Binding(ref binding) => {
                                collect_lifetimes(&binding.ty, out);
                            }
                            syn::GenericArgument::Const(_) => {}
                        }
                    }
                }
            }
        }
        syn::Type::Paren(ref ty) => {
            collect_lifetimes(&ty.elem, out);
        }
        syn::Type::Group(ref ty) => {
            collect_lifetimes(&ty.elem, out);
        }
        syn::Type::BareFn(_)
        | syn::Type::Never(_)
        | syn::Type::TraitObject(_)
        | syn::Type::ImplTrait(_)
        | syn::Type::Infer(_)
        | syn::Type::Macro(_)
        | syn::Type::Verbatim(_) => {}
    }
}

fn parse_lit_str<T>(s: &syn::LitStr) -> Result<T, ParseError>
where
    T: Synom,
{
    let tokens = try!(spanned_tokens(s));
    syn::parse2(tokens)
}

fn spanned_tokens(s: &syn::LitStr) -> Result<TokenStream, ParseError> {
    let stream = try!(syn::parse_str(&s.value()));
    Ok(respan_token_stream(stream, s.span()))
}

fn respan_token_stream(stream: TokenStream, span: Span) -> TokenStream {
    stream
        .into_iter()
        .map(|token| respan_token_tree(token, span))
        .collect()
}

fn respan_token_tree(mut token: TokenTree, span: Span) -> TokenTree {
    if let TokenTree::Group(ref mut g) = token {
        *g = Group::new(g.delimiter(), respan_token_stream(g.stream().clone(), span));
    }
    token.set_span(span);
    token
}
