// Copyright 2017 Serde Developers
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use Ctxt;
use syn;
use syn::MetaItem::{List, NameValue, Word};
use syn::NestedMetaItem::{Literal, MetaItem};
use synom::IResult;
use std::collections::BTreeSet;
use std::str::FromStr;

// This module handles parsing of `#[serde(...)]` attributes. The entrypoints
// are `attr::Container::from_ast`, `attr::Variant::from_ast`, and
// `attr::Field::from_ast`. Each returns an instance of the corresponding
// struct. Note that none of them return a Result. Unrecognized, malformed, or
// duplicated attributes result in a span_err but otherwise are ignored. The
// user will see errors simultaneously for all bad attributes in the crate
// rather than just the first.

pub use case::RenameRule;

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

#[derive(Debug)]
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
#[derive(Debug)]
pub struct Container {
    name: Name,
    deny_unknown_fields: bool,
    default: Default,
    rename_all: RenameRule,
    ser_bound: Option<Vec<syn::WherePredicate>>,
    de_bound: Option<Vec<syn::WherePredicate>>,
    tag: EnumTag,
    from_type: Option<syn::Ty>,
    into_type: Option<syn::Ty>,
    remote: Option<syn::Path>,
    identifier: Identifier,
}

/// Styles of representing an enum.
#[derive(Debug)]
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
#[derive(Copy, Clone, Debug)]
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
        let mut from_type = Attr::none(cx, "from");
        let mut into_type = Attr::none(cx, "into");
        let mut remote = Attr::none(cx, "remote");
        let mut field_identifier = BoolAttr::none(cx, "field_identifier");
        let mut variant_identifier = BoolAttr::none(cx, "variant_identifier");

        for meta_items in item.attrs.iter().filter_map(get_serde_meta_items) {
            for meta_item in meta_items {
                match meta_item {
                    // Parse `#[serde(rename = "foo")]`
                    MetaItem(NameValue(ref name, ref lit)) if name == "rename" => {
                        if let Ok(s) = get_string_from_lit(cx, name.as_ref(), name.as_ref(), lit) {
                            ser_name.set(s.clone());
                            de_name.set(s);
                        }
                    }

                    // Parse `#[serde(rename(serialize = "foo", deserialize = "bar"))]`
                    MetaItem(List(ref name, ref meta_items)) if name == "rename" => {
                        if let Ok((ser, de)) = get_renames(cx, meta_items) {
                            ser_name.set_opt(ser);
                            de_name.set_opt(de);
                        }
                    }

                    // Parse `#[serde(rename_all = "foo")]`
                    MetaItem(NameValue(ref name, ref lit)) if name == "rename_all" => {
                        if let Ok(s) = get_string_from_lit(cx, name.as_ref(), name.as_ref(), lit) {
                            match RenameRule::from_str(&s) {
                                Ok(rename_rule) => rename_all.set(rename_rule),
                                Err(()) => cx.error(format!(
                                    "unknown rename rule for #[serde(rename_all \
                                     = {:?})]",
                                    s
                                )),
                            }
                        }
                    }

                    // Parse `#[serde(deny_unknown_fields)]`
                    MetaItem(Word(ref name)) if name == "deny_unknown_fields" => {
                        deny_unknown_fields.set_true();
                    }

                    // Parse `#[serde(default)]`
                    MetaItem(Word(ref name)) if name == "default" => match item.body {
                        syn::Body::Struct(syn::VariantData::Struct(_)) => {
                            default.set(Default::Default);
                        }
                        _ => cx.error(
                            "#[serde(default)] can only be used on structs \
                             with named fields",
                        ),
                    },

                    // Parse `#[serde(default = "...")]`
                    MetaItem(NameValue(ref name, ref lit)) if name == "default" => {
                        if let Ok(path) = parse_lit_into_path(cx, name.as_ref(), lit) {
                            match item.body {
                                syn::Body::Struct(syn::VariantData::Struct(_)) => {
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
                    MetaItem(NameValue(ref name, ref lit)) if name == "bound" => {
                        if let Ok(where_predicates) =
                            parse_lit_into_where(cx, name.as_ref(), name.as_ref(), lit)
                        {
                            ser_bound.set(where_predicates.clone());
                            de_bound.set(where_predicates);
                        }
                    }

                    // Parse `#[serde(bound(serialize = "D: Serialize", deserialize = "D: Deserialize"))]`
                    MetaItem(List(ref name, ref meta_items)) if name == "bound" => {
                        if let Ok((ser, de)) = get_where_predicates(cx, meta_items) {
                            ser_bound.set_opt(ser);
                            de_bound.set_opt(de);
                        }
                    }

                    // Parse `#[serde(untagged)]`
                    MetaItem(Word(ref name)) if name == "untagged" => match item.body {
                        syn::Body::Enum(_) => {
                            untagged.set_true();
                        }
                        syn::Body::Struct(_) => {
                            cx.error("#[serde(untagged)] can only be used on enums")
                        }
                    },

                    // Parse `#[serde(tag = "type")]`
                    MetaItem(NameValue(ref name, ref lit)) if name == "tag" => {
                        if let Ok(s) = get_string_from_lit(cx, name.as_ref(), name.as_ref(), lit) {
                            match item.body {
                                syn::Body::Enum(_) => {
                                    internal_tag.set(s);
                                }
                                syn::Body::Struct(_) => {
                                    cx.error("#[serde(tag = \"...\")] can only be used on enums")
                                }
                            }
                        }
                    }

                    // Parse `#[serde(content = "c")]`
                    MetaItem(NameValue(ref name, ref lit)) if name == "content" => {
                        if let Ok(s) = get_string_from_lit(cx, name.as_ref(), name.as_ref(), lit) {
                            match item.body {
                                syn::Body::Enum(_) => {
                                    content.set(s);
                                }
                                syn::Body::Struct(_) => cx.error(
                                    "#[serde(content = \"...\")] can only be used on \
                                     enums",
                                ),
                            }
                        }
                    }

                    // Parse `#[serde(from = "Type")]
                    MetaItem(NameValue(ref name, ref lit)) if name == "from" => {
                        if let Ok(from_ty) = parse_lit_into_ty(cx, name.as_ref(), lit) {
                            from_type.set_opt(Some(from_ty));
                        }
                    }

                    // Parse `#[serde(into = "Type")]
                    MetaItem(NameValue(ref name, ref lit)) if name == "into" => {
                        if let Ok(into_ty) = parse_lit_into_ty(cx, name.as_ref(), lit) {
                            into_type.set_opt(Some(into_ty));
                        }
                    }

                    // Parse `#[serde(remote = "...")]`
                    MetaItem(NameValue(ref name, ref lit)) if name == "remote" => {
                        if let Ok(path) = parse_lit_into_path(cx, name.as_ref(), lit) {
                            remote.set(path);
                        }
                    }

                    // Parse `#[serde(field_identifier)]`
                    MetaItem(Word(ref name)) if name == "field_identifier" => {
                        field_identifier.set_true();
                    }

                    // Parse `#[serde(variant_identifier)]`
                    MetaItem(Word(ref name)) if name == "variant_identifier" => {
                        variant_identifier.set_true();
                    }

                    MetaItem(ref meta_item) => {
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
            tag: decide_tag(cx, item, untagged, internal_tag, content),
            from_type: from_type.get(),
            into_type: into_type.get(),
            remote: remote.get(),
            identifier: decide_identifier(cx, item, field_identifier, variant_identifier),
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

    pub fn from_type(&self) -> Option<&syn::Ty> {
        self.from_type.as_ref()
    }

    pub fn into_type(&self) -> Option<&syn::Ty> {
        self.into_type.as_ref()
    }

    pub fn remote(&self) -> Option<&syn::Path> {
        self.remote.as_ref()
    }

    pub fn identifier(&self) -> Identifier {
        self.identifier
    }
}

fn decide_tag(
    cx: &Ctxt,
    item: &syn::DeriveInput,
    untagged: BoolAttr,
    internal_tag: Attr<String>,
    content: Attr<String>,
) -> EnumTag {
    match (untagged.get(), internal_tag.get(), content.get()) {
        (false, None, None) => EnumTag::External,
        (true, None, None) => EnumTag::None,
        (false, Some(tag), None) => {
            // Check that there are no tuple variants.
            if let syn::Body::Enum(ref variants) = item.body {
                for variant in variants {
                    match variant.data {
                        syn::VariantData::Struct(_) | syn::VariantData::Unit => {}
                        syn::VariantData::Tuple(ref fields) => {
                            if fields.len() != 1 {
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
    field_identifier: BoolAttr,
    variant_identifier: BoolAttr,
) -> Identifier {
    match (&item.body, field_identifier.get(), variant_identifier.get()) {
        (_, false, false) => Identifier::No,
        (_, true, true) => {
            cx.error("`field_identifier` and `variant_identifier` cannot both be set");
            Identifier::No
        }
        (&syn::Body::Struct(_), true, false) => {
            cx.error("`field_identifier` can only be used on an enum");
            Identifier::No
        }
        (&syn::Body::Struct(_), false, true) => {
            cx.error("`variant_identifier` can only be used on an enum");
            Identifier::No
        }
        (&syn::Body::Enum(_), true, false) => Identifier::Field,
        (&syn::Body::Enum(_), false, true) => Identifier::Variant,
    }
}

/// Represents variant attribute information
#[derive(Debug)]
pub struct Variant {
    name: Name,
    ser_renamed: bool,
    de_renamed: bool,
    rename_all: RenameRule,
    skip_deserializing: bool,
    skip_serializing: bool,
    other: bool,
    serialize_with: Option<syn::Path>,
    deserialize_with: Option<syn::Path>,
    borrow: Option<syn::MetaItem>,
}

impl Variant {
    pub fn from_ast(cx: &Ctxt, variant: &syn::Variant) -> Self {
        let mut ser_name = Attr::none(cx, "rename");
        let mut de_name = Attr::none(cx, "rename");
        let mut skip_deserializing = BoolAttr::none(cx, "skip_deserializing");
        let mut skip_serializing = BoolAttr::none(cx, "skip_serializing");
        let mut rename_all = Attr::none(cx, "rename_all");
        let mut other = BoolAttr::none(cx, "other");
        let mut serialize_with = Attr::none(cx, "serialize_with");
        let mut deserialize_with = Attr::none(cx, "deserialize_with");
        let mut borrow = Attr::none(cx, "borrow");

        for meta_items in variant.attrs.iter().filter_map(get_serde_meta_items) {
            for meta_item in meta_items {
                match meta_item {
                    // Parse `#[serde(rename = "foo")]`
                    MetaItem(NameValue(ref name, ref lit)) if name == "rename" => {
                        if let Ok(s) = get_string_from_lit(cx, name.as_ref(), name.as_ref(), lit) {
                            ser_name.set(s.clone());
                            de_name.set(s);
                        }
                    }

                    // Parse `#[serde(rename(serialize = "foo", deserialize = "bar"))]`
                    MetaItem(List(ref name, ref meta_items)) if name == "rename" => {
                        if let Ok((ser, de)) = get_renames(cx, meta_items) {
                            ser_name.set_opt(ser);
                            de_name.set_opt(de);
                        }
                    }

                    // Parse `#[serde(rename_all = "foo")]`
                    MetaItem(NameValue(ref name, ref lit)) if name == "rename_all" => {
                        if let Ok(s) = get_string_from_lit(cx, name.as_ref(), name.as_ref(), lit) {
                            match RenameRule::from_str(&s) {
                                Ok(rename_rule) => rename_all.set(rename_rule),
                                Err(()) => cx.error(format!(
                                    "unknown rename rule for #[serde(rename_all \
                                     = {:?})]",
                                    s
                                )),
                            }
                        }
                    }

                    // Parse `#[serde(skip_deserializing)]`
                    MetaItem(Word(ref name)) if name == "skip_deserializing" => {
                        skip_deserializing.set_true();
                    }

                    // Parse `#[serde(skip_serializing)]`
                    MetaItem(Word(ref name)) if name == "skip_serializing" => {
                        skip_serializing.set_true();
                    }

                    // Parse `#[serde(other)]`
                    MetaItem(Word(ref name)) if name == "other" => {
                        other.set_true();
                    }

                    // Parse `#[serde(with = "...")]`
                    MetaItem(NameValue(ref name, ref lit)) if name == "with" => {
                        if let Ok(path) = parse_lit_into_path(cx, name.as_ref(), lit) {
                            let mut ser_path = path.clone();
                            ser_path.segments.push("serialize".into());
                            serialize_with.set(ser_path);
                            let mut de_path = path;
                            de_path.segments.push("deserialize".into());
                            deserialize_with.set(de_path);
                        }
                    }

                    // Parse `#[serde(serialize_with = "...")]`
                    MetaItem(NameValue(ref name, ref lit)) if name == "serialize_with" => {
                        if let Ok(path) = parse_lit_into_path(cx, name.as_ref(), lit) {
                            serialize_with.set(path);
                        }
                    }

                    // Parse `#[serde(deserialize_with = "...")]`
                    MetaItem(NameValue(ref name, ref lit)) if name == "deserialize_with" => {
                        if let Ok(path) = parse_lit_into_path(cx, name.as_ref(), lit) {
                            deserialize_with.set(path);
                        }
                    }

                    // Defer `#[serde(borrow)]` and `#[serde(borrow = "'a + 'b")]`
                    MetaItem(ref mi) if mi.name() == "borrow" => match variant.data {
                        syn::VariantData::Tuple(ref fields) if fields.len() == 1 => {
                            borrow.set(mi.clone());
                        }
                        _ => {
                            cx.error("#[serde(borrow)] may only be used on newtype variants");
                        }
                    },

                    MetaItem(ref meta_item) => {
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

    pub fn skip_deserializing(&self) -> bool {
        self.skip_deserializing
    }

    pub fn skip_serializing(&self) -> bool {
        self.skip_serializing
    }

    pub fn other(&self) -> bool {
        self.other
    }

    pub fn serialize_with(&self) -> Option<&syn::Path> {
        self.serialize_with.as_ref()
    }

    pub fn deserialize_with(&self) -> Option<&syn::Path> {
        self.deserialize_with.as_ref()
    }
}

/// Represents field attribute information
#[derive(Debug)]
pub struct Field {
    name: Name,
    ser_renamed: bool,
    de_renamed: bool,
    skip_serializing: bool,
    skip_deserializing: bool,
    skip_serializing_if: Option<syn::Path>,
    default: Default,
    serialize_with: Option<syn::Path>,
    deserialize_with: Option<syn::Path>,
    ser_bound: Option<Vec<syn::WherePredicate>>,
    de_bound: Option<Vec<syn::WherePredicate>>,
    borrowed_lifetimes: BTreeSet<syn::Lifetime>,
    getter: Option<syn::Path>,
}

/// Represents the default to use for a field when deserializing.
#[derive(Debug, PartialEq)]
pub enum Default {
    /// Field must always be specified because it does not have a default.
    None,
    /// The default is given by `std::default::Default::default()`.
    Default,
    /// The default is given by this function.
    Path(syn::Path),
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

        let ident = match field.ident {
            Some(ref ident) => ident.to_string(),
            None => index.to_string(),
        };

        let variant_borrow = attrs
            .map(|variant| &variant.borrow)
            .unwrap_or(&None)
            .as_ref()
            .map(|borrow| vec![MetaItem(borrow.clone())]);

        for meta_items in field
            .attrs
            .iter()
            .filter_map(get_serde_meta_items)
            .chain(variant_borrow)
        {
            for meta_item in meta_items {
                match meta_item {
                    // Parse `#[serde(rename = "foo")]`
                    MetaItem(NameValue(ref name, ref lit)) if name == "rename" => {
                        if let Ok(s) = get_string_from_lit(cx, name.as_ref(), name.as_ref(), lit) {
                            ser_name.set(s.clone());
                            de_name.set(s);
                        }
                    }

                    // Parse `#[serde(rename(serialize = "foo", deserialize = "bar"))]`
                    MetaItem(List(ref name, ref meta_items)) if name == "rename" => {
                        if let Ok((ser, de)) = get_renames(cx, meta_items) {
                            ser_name.set_opt(ser);
                            de_name.set_opt(de);
                        }
                    }

                    // Parse `#[serde(default)]`
                    MetaItem(Word(ref name)) if name == "default" => {
                        default.set(Default::Default);
                    }

                    // Parse `#[serde(default = "...")]`
                    MetaItem(NameValue(ref name, ref lit)) if name == "default" => {
                        if let Ok(path) = parse_lit_into_path(cx, name.as_ref(), lit) {
                            default.set(Default::Path(path));
                        }
                    }

                    // Parse `#[serde(skip_serializing)]`
                    MetaItem(Word(ref name)) if name == "skip_serializing" => {
                        skip_serializing.set_true();
                    }

                    // Parse `#[serde(skip_deserializing)]`
                    MetaItem(Word(ref name)) if name == "skip_deserializing" => {
                        skip_deserializing.set_true();
                    }

                    // Parse `#[serde(skip)]`
                    MetaItem(Word(ref name)) if name == "skip" => {
                        skip_serializing.set_true();
                        skip_deserializing.set_true();
                    }

                    // Parse `#[serde(skip_serializing_if = "...")]`
                    MetaItem(NameValue(ref name, ref lit)) if name == "skip_serializing_if" => {
                        if let Ok(path) = parse_lit_into_path(cx, name.as_ref(), lit) {
                            skip_serializing_if.set(path);
                        }
                    }

                    // Parse `#[serde(serialize_with = "...")]`
                    MetaItem(NameValue(ref name, ref lit)) if name == "serialize_with" => {
                        if let Ok(path) = parse_lit_into_path(cx, name.as_ref(), lit) {
                            serialize_with.set(path);
                        }
                    }

                    // Parse `#[serde(deserialize_with = "...")]`
                    MetaItem(NameValue(ref name, ref lit)) if name == "deserialize_with" => {
                        if let Ok(path) = parse_lit_into_path(cx, name.as_ref(), lit) {
                            deserialize_with.set(path);
                        }
                    }

                    // Parse `#[serde(with = "...")]`
                    MetaItem(NameValue(ref name, ref lit)) if name == "with" => {
                        if let Ok(path) = parse_lit_into_path(cx, name.as_ref(), lit) {
                            let mut ser_path = path.clone();
                            ser_path.segments.push("serialize".into());
                            serialize_with.set(ser_path);
                            let mut de_path = path;
                            de_path.segments.push("deserialize".into());
                            deserialize_with.set(de_path);
                        }
                    }

                    // Parse `#[serde(bound = "D: Serialize")]`
                    MetaItem(NameValue(ref name, ref lit)) if name == "bound" => {
                        if let Ok(where_predicates) =
                            parse_lit_into_where(cx, name.as_ref(), name.as_ref(), lit)
                        {
                            ser_bound.set(where_predicates.clone());
                            de_bound.set(where_predicates);
                        }
                    }

                    // Parse `#[serde(bound(serialize = "D: Serialize", deserialize = "D: Deserialize"))]`
                    MetaItem(List(ref name, ref meta_items)) if name == "bound" => {
                        if let Ok((ser, de)) = get_where_predicates(cx, meta_items) {
                            ser_bound.set_opt(ser);
                            de_bound.set_opt(de);
                        }
                    }

                    // Parse `#[serde(borrow)]`
                    MetaItem(Word(ref name)) if name == "borrow" => {
                        if let Ok(borrowable) = borrowable_lifetimes(cx, &ident, &field.ty) {
                            borrowed_lifetimes.set(borrowable);
                        }
                    }

                    // Parse `#[serde(borrow = "'a + 'b")]`
                    MetaItem(NameValue(ref name, ref lit)) if name == "borrow" => {
                        if let Ok(lifetimes) = parse_lit_into_lifetimes(cx, name.as_ref(), lit) {
                            if let Ok(borrowable) = borrowable_lifetimes(cx, &ident, &field.ty) {
                                for lifetime in &lifetimes {
                                    if !borrowable.contains(lifetime) {
                                        cx.error(format!(
                                            "field `{}` does not have lifetime {}",
                                            ident, lifetime.ident
                                        ));
                                    }
                                }
                                borrowed_lifetimes.set(lifetimes);
                            }
                        }
                    }

                    // Parse `#[serde(getter = "...")]`
                    MetaItem(NameValue(ref name, ref lit)) if name == "getter" => {
                        if let Ok(path) = parse_lit_into_path(cx, name.as_ref(), lit) {
                            getter.set(path);
                        }
                    }

                    MetaItem(ref meta_item) => {
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
        if container_default == &Default::None && skip_deserializing.0.value.is_some() {
            default.set_if_none(Default::Default);
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
            if is_cow(&field.ty, "str") {
                let path = syn::parse_path("_serde::private::de::borrow_cow_str").unwrap();
                deserialize_with.set_if_none(path);
            } else if is_cow(&field.ty, "[u8]") {
                let path = syn::parse_path("_serde::private::de::borrow_cow_bytes").unwrap();
                deserialize_with.set_if_none(path);
            }
        } else if is_rptr(&field.ty, "str") || is_rptr(&field.ty, "[u8]") {
            // Types &str and &[u8] are always implicitly borrowed. No need for
            // a #[serde(borrow)].
            borrowed_lifetimes = borrowable_lifetimes(cx, &ident, &field.ty).unwrap();
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

    pub fn skip_serializing_if(&self) -> Option<&syn::Path> {
        self.skip_serializing_if.as_ref()
    }

    pub fn default(&self) -> &Default {
        &self.default
    }

    pub fn serialize_with(&self) -> Option<&syn::Path> {
        self.serialize_with.as_ref()
    }

    pub fn deserialize_with(&self) -> Option<&syn::Path> {
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

    pub fn getter(&self) -> Option<&syn::Path> {
        self.getter.as_ref()
    }
}

type SerAndDe<T> = (Option<T>, Option<T>);

fn get_ser_and_de<T, F>(
    cx: &Ctxt,
    attr_name: &'static str,
    items: &[syn::NestedMetaItem],
    f: F,
) -> Result<SerAndDe<T>, ()>
where
    F: Fn(&Ctxt, &str, &str, &syn::Lit) -> Result<T, ()>,
{
    let mut ser_item = Attr::none(cx, attr_name);
    let mut de_item = Attr::none(cx, attr_name);

    for item in items {
        match *item {
            MetaItem(NameValue(ref name, ref lit)) if name == "serialize" => {
                if let Ok(v) = f(cx, attr_name, name.as_ref(), lit) {
                    ser_item.set(v);
                }
            }

            MetaItem(NameValue(ref name, ref lit)) if name == "deserialize" => {
                if let Ok(v) = f(cx, attr_name, name.as_ref(), lit) {
                    de_item.set(v);
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

    Ok((ser_item.get(), de_item.get()))
}

fn get_renames(cx: &Ctxt, items: &[syn::NestedMetaItem]) -> Result<SerAndDe<String>, ()> {
    get_ser_and_de(cx, "rename", items, get_string_from_lit)
}

fn get_where_predicates(
    cx: &Ctxt,
    items: &[syn::NestedMetaItem],
) -> Result<SerAndDe<Vec<syn::WherePredicate>>, ()> {
    get_ser_and_de(cx, "bound", items, parse_lit_into_where)
}

pub fn get_serde_meta_items(attr: &syn::Attribute) -> Option<Vec<syn::NestedMetaItem>> {
    match attr.value {
        List(ref name, ref items) if name == "serde" => Some(items.iter().cloned().collect()),
        _ => None,
    }
}

fn get_string_from_lit(
    cx: &Ctxt,
    attr_name: &str,
    meta_item_name: &str,
    lit: &syn::Lit,
) -> Result<String, ()> {
    if let syn::Lit::Str(ref s, _) = *lit {
        Ok(s.clone())
    } else {
        cx.error(format!(
            "expected serde {} attribute to be a string: `{} = \"...\"`",
            attr_name, meta_item_name
        ));
        Err(())
    }
}

fn parse_lit_into_path(cx: &Ctxt, attr_name: &str, lit: &syn::Lit) -> Result<syn::Path, ()> {
    let string = try!(get_string_from_lit(cx, attr_name, attr_name, lit));
    syn::parse_path(&string).map_err(|err| cx.error(err))
}

fn parse_lit_into_where(
    cx: &Ctxt,
    attr_name: &str,
    meta_item_name: &str,
    lit: &syn::Lit,
) -> Result<Vec<syn::WherePredicate>, ()> {
    let string = try!(get_string_from_lit(cx, attr_name, meta_item_name, lit));
    if string.is_empty() {
        return Ok(Vec::new());
    }

    let where_string = format!("where {}", string);

    syn::parse_where_clause(&where_string)
        .map(|wh| wh.predicates)
        .map_err(|err| cx.error(err))
}

fn parse_lit_into_ty(cx: &Ctxt, attr_name: &str, lit: &syn::Lit) -> Result<syn::Ty, ()> {
    let string = try!(get_string_from_lit(cx, attr_name, attr_name, lit));

    syn::parse_type(&string).map_err(|_| {
        cx.error(format!(
            "failed to parse type: {} = {:?}",
            attr_name, string
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
    let string = try!(get_string_from_lit(cx, attr_name, attr_name, lit));
    if string.is_empty() {
        cx.error("at least one lifetime must be borrowed");
        return Err(());
    }

    named!(lifetimes -> Vec<syn::Lifetime>,
        separated_nonempty_list!(punct!("+"), syn::parse::lifetime)
    );

    if let IResult::Done(rest, o) = lifetimes(&string) {
        if rest.trim().is_empty() {
            let mut set = BTreeSet::new();
            for lifetime in o {
                if !set.insert(lifetime.clone()) {
                    cx.error(format!("duplicate borrowed lifetime `{}`", lifetime.ident));
                }
            }
            return Ok(set);
        }
    }
    Err(cx.error(format!("failed to parse borrowed lifetimes: {:?}", string)))
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
fn is_cow(ty: &syn::Ty, elem: &str) -> bool {
    let path = match *ty {
        syn::Ty::Path(None, ref path) => path,
        _ => {
            return false;
        }
    };
    let seg = match path.segments.last() {
        Some(seg) => seg,
        None => {
            return false;
        }
    };
    let params = match seg.parameters {
        syn::PathParameters::AngleBracketed(ref params) => params,
        _ => {
            return false;
        }
    };
    seg.ident == "Cow" && params.lifetimes.len() == 1
        && params.types == vec![syn::parse_type(elem).unwrap()] && params.bindings.is_empty()
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
fn is_rptr(ty: &syn::Ty, elem: &str) -> bool {
    match *ty {
        syn::Ty::Rptr(Some(_), ref mut_ty) => {
            mut_ty.mutability == syn::Mutability::Immutable
                && mut_ty.ty == syn::parse_type(elem).unwrap()
        }
        _ => false,
    }
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
    ty: &syn::Ty,
) -> Result<BTreeSet<syn::Lifetime>, ()> {
    let mut lifetimes = BTreeSet::new();
    collect_lifetimes(ty, &mut lifetimes);
    if lifetimes.is_empty() {
        Err(cx.error(format!("field `{}` has no lifetimes to borrow", name)))
    } else {
        Ok(lifetimes)
    }
}

fn collect_lifetimes(ty: &syn::Ty, out: &mut BTreeSet<syn::Lifetime>) {
    match *ty {
        syn::Ty::Slice(ref elem) | syn::Ty::Array(ref elem, _) | syn::Ty::Paren(ref elem) => {
            collect_lifetimes(elem, out);
        }
        syn::Ty::Ptr(ref elem) => {
            collect_lifetimes(&elem.ty, out);
        }
        syn::Ty::Rptr(ref lifetime, ref elem) => {
            out.extend(lifetime.iter().cloned());
            collect_lifetimes(&elem.ty, out);
        }
        syn::Ty::Tup(ref elems) => for elem in elems {
            collect_lifetimes(elem, out);
        },
        syn::Ty::Path(ref qself, ref path) => {
            if let Some(ref qself) = *qself {
                collect_lifetimes(&qself.ty, out);
            }
            for seg in &path.segments {
                if let syn::PathParameters::AngleBracketed(ref params) = seg.parameters {
                    out.extend(params.lifetimes.iter().cloned());
                    for ty in &params.types {
                        collect_lifetimes(ty, out);
                    }
                    for binding in &params.bindings {
                        collect_lifetimes(&binding.ty, out);
                    }
                }
            }
        }
        syn::Ty::BareFn(_)
        | syn::Ty::Never
        | syn::Ty::TraitObject(_)
        | syn::Ty::ImplTrait(_)
        | syn::Ty::Infer
        | syn::Ty::Mac(_) => {}
    }
}
