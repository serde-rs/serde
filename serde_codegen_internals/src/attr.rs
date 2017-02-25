use Ctxt;
use syn;
use syn::MetaItem::{List, NameValue, Word};
use syn::NestedMetaItem::{Literal, MetaItem};
use std::str::FromStr;

// This module handles parsing of `#[serde(...)]` attributes. The entrypoints
// are `attr::Item::from_ast`, `attr::Variant::from_ast`, and
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
            self.cx.error(format!("duplicate serde attribute `{}`", self.name));
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
pub struct Item {
    name: Name,
    deny_unknown_fields: bool,
    default: Default,
    rename_all: RenameRule,
    ser_bound: Option<Vec<syn::WherePredicate>>,
    de_bound: Option<Vec<syn::WherePredicate>>,
    tag: EnumTag,
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

impl Item {
    /// Extract out the `#[serde(...)]` attributes from an item.
    pub fn from_ast(cx: &Ctxt, item: &syn::MacroInput) -> Self {
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

        for meta_items in item.attrs.iter().filter_map(get_serde_meta_items) {
            for meta_item in meta_items {
                match meta_item {
                    // Parse `#[serde(rename="foo")]`
                    MetaItem(NameValue(ref name, ref lit)) if name == "rename" => {
                        if let Ok(s) = get_string_from_lit(cx, name.as_ref(), name.as_ref(), lit) {
                            ser_name.set(s.clone());
                            de_name.set(s);
                        }
                    }

                    // Parse `#[serde(rename(serialize="foo", deserialize="bar"))]`
                    MetaItem(List(ref name, ref meta_items)) if name == "rename" => {
                        if let Ok((ser, de)) = get_renames(cx, meta_items) {
                            ser_name.set_opt(ser);
                            de_name.set_opt(de);
                        }
                    }

                    // Parse `#[serde(rename_all="foo")]`
                    MetaItem(NameValue(ref name, ref lit)) if name == "rename_all" => {
                        if let Ok(s) = get_string_from_lit(cx, name.as_ref(), name.as_ref(), lit) {
                            match RenameRule::from_str(&s) {
                                Ok(rename_rule) => rename_all.set(rename_rule),
                                Err(()) => {
                                    cx.error(format!("unknown rename rule for #[serde(rename_all \
                                                      = {:?})]",
                                                     s))
                                }
                            }
                        }
                    }

                    // Parse `#[serde(deny_unknown_fields)]`
                    MetaItem(Word(ref name)) if name == "deny_unknown_fields" => {
                        deny_unknown_fields.set_true();
                    }

                    // Parse `#[serde(default)]`
                    MetaItem(Word(ref name)) if name == "default" => {
                        match item.body {
                            syn::Body::Struct(syn::VariantData::Struct(_)) => {
                                default.set(Default::Default);
                            }
                            _ => {
                                cx.error("#[serde(default)] can only be used on structs \
                                          with named fields")
                            }
                        }
                    }

                    // Parse `#[serde(default="...")]`
                    MetaItem(NameValue(ref name, ref lit)) if name == "default" => {
                        if let Ok(path) = parse_lit_into_path(cx, name.as_ref(), lit) {
                            match item.body {
                                syn::Body::Struct(syn::VariantData::Struct(_)) => {
                                    default.set(Default::Path(path));
                                }
                                _ => {
                                    cx.error("#[serde(default = \"...\")] can only be used \
                                              on structs with named fields")
                                }
                            }
                        }
                    }

                    // Parse `#[serde(bound="D: Serialize")]`
                    MetaItem(NameValue(ref name, ref lit)) if name == "bound" => {
                        if let Ok(where_predicates) =
                            parse_lit_into_where(cx, name.as_ref(), name.as_ref(), lit) {
                            ser_bound.set(where_predicates.clone());
                            de_bound.set(where_predicates);
                        }
                    }

                    // Parse `#[serde(bound(serialize="D: Serialize", deserialize="D: Deserialize"))]`
                    MetaItem(List(ref name, ref meta_items)) if name == "bound" => {
                        if let Ok((ser, de)) = get_where_predicates(cx, meta_items) {
                            ser_bound.set_opt(ser);
                            de_bound.set_opt(de);
                        }
                    }

                    // Parse `#[serde(untagged)]`
                    MetaItem(Word(ref name)) if name == "untagged" => {
                        match item.body {
                            syn::Body::Enum(_) => {
                                untagged.set_true();
                            }
                            syn::Body::Struct(_) => {
                                cx.error("#[serde(untagged)] can only be used on enums")
                            }
                        }
                    }

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
                                syn::Body::Struct(_) => {
                                    cx.error("#[serde(content = \"...\")] can only be used on \
                                              enums")
                                }
                            }
                        }
                    }

                    MetaItem(ref meta_item) => {
                        cx.error(format!("unknown serde container attribute `{}`",
                                         meta_item.name()));
                    }

                    Literal(_) => {
                        cx.error("unexpected literal in serde container attribute");
                    }
                }
            }
        }

        let tag = match (untagged.get(), internal_tag.get(), content.get()) {
            (false, None, None) => EnumTag::External,
            (true, None, None) => EnumTag::None,
            (false, Some(tag), None) => {
                // Check that there are no tuple variants.
                if let syn::Body::Enum(ref variants) = item.body {
                    for variant in variants {
                        match variant.data {
                            syn::VariantData::Struct(_) |
                            syn::VariantData::Unit => {}
                            syn::VariantData::Tuple(ref fields) => {
                                if fields.len() != 1 {
                                    cx.error("#[serde(tag = \"...\")] cannot be used with tuple \
                                              variants");
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
            (false, Some(tag), Some(content)) => {
                EnumTag::Adjacent {
                    tag: tag,
                    content: content,
                }
            }
            (true, Some(_), Some(_)) => {
                cx.error("untagged enum cannot have #[serde(tag = \"...\", content = \"...\")]");
                EnumTag::External
            }
        };

        Item {
            name: Name {
                serialize: ser_name.get().unwrap_or_else(|| item.ident.to_string()),
                deserialize: de_name.get().unwrap_or_else(|| item.ident.to_string()),
            },
            deny_unknown_fields: deny_unknown_fields.get(),
            default: default.get().unwrap_or(Default::None),
            rename_all: rename_all.get().unwrap_or(RenameRule::None),
            ser_bound: ser_bound.get(),
            de_bound: de_bound.get(),
            tag: tag,
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
}

impl Variant {
    pub fn from_ast(cx: &Ctxt, variant: &syn::Variant) -> Self {
        let mut ser_name = Attr::none(cx, "rename");
        let mut de_name = Attr::none(cx, "rename");
        let mut skip_deserializing = BoolAttr::none(cx, "skip_deserializing");
        let mut skip_serializing = BoolAttr::none(cx, "skip_serializing");
        let mut rename_all = Attr::none(cx, "rename_all");

        for meta_items in variant.attrs.iter().filter_map(get_serde_meta_items) {
            for meta_item in meta_items {
                match meta_item {
                    // Parse `#[serde(rename="foo")]`
                    MetaItem(NameValue(ref name, ref lit)) if name == "rename" => {
                        if let Ok(s) = get_string_from_lit(cx, name.as_ref(), name.as_ref(), lit) {
                            ser_name.set(s.clone());
                            de_name.set(s);
                        }
                    }

                    // Parse `#[serde(rename(serialize="foo", deserialize="bar"))]`
                    MetaItem(List(ref name, ref meta_items)) if name == "rename" => {
                        if let Ok((ser, de)) = get_renames(cx, meta_items) {
                            ser_name.set_opt(ser);
                            de_name.set_opt(de);
                        }
                    }

                    // Parse `#[serde(rename_all="foo")]`
                    MetaItem(NameValue(ref name, ref lit)) if name == "rename_all" => {
                        if let Ok(s) = get_string_from_lit(cx, name.as_ref(), name.as_ref(), lit) {
                            match RenameRule::from_str(&s) {
                                Ok(rename_rule) => rename_all.set(rename_rule),
                                Err(()) => {
                                    cx.error(format!("unknown rename rule for #[serde(rename_all \
                                                      = {:?})]",
                                                     s))
                                }
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

                    MetaItem(ref meta_item) => {
                        cx.error(format!("unknown serde variant attribute `{}`", meta_item.name()));
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
    pub fn from_ast(cx: &Ctxt, index: usize, field: &syn::Field) -> Self {
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
            Some(ref ident) => ident.to_string(),
            None => index.to_string(),
        };

        for meta_items in field.attrs.iter().filter_map(get_serde_meta_items) {
            for meta_item in meta_items {
                match meta_item {
                    // Parse `#[serde(rename="foo")]`
                    MetaItem(NameValue(ref name, ref lit)) if name == "rename" => {
                        if let Ok(s) = get_string_from_lit(cx, name.as_ref(), name.as_ref(), lit) {
                            ser_name.set(s.clone());
                            de_name.set(s);
                        }
                    }

                    // Parse `#[serde(rename(serialize="foo", deserialize="bar"))]`
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

                    // Parse `#[serde(default="...")]`
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

                    // Parse `#[serde(skip_serializing_if="...")]`
                    MetaItem(NameValue(ref name, ref lit)) if name == "skip_serializing_if" => {
                        if let Ok(path) = parse_lit_into_path(cx, name.as_ref(), lit) {
                            skip_serializing_if.set(path);
                        }
                    }

                    // Parse `#[serde(serialize_with="...")]`
                    MetaItem(NameValue(ref name, ref lit)) if name == "serialize_with" => {
                        if let Ok(path) = parse_lit_into_path(cx, name.as_ref(), lit) {
                            serialize_with.set(path);
                        }
                    }

                    // Parse `#[serde(deserialize_with="...")]`
                    MetaItem(NameValue(ref name, ref lit)) if name == "deserialize_with" => {
                        if let Ok(path) = parse_lit_into_path(cx, name.as_ref(), lit) {
                            deserialize_with.set(path);
                        }
                    }

                    // Parse `#[serde(with="...")]`
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

                    // Parse `#[serde(bound="D: Serialize")]`
                    MetaItem(NameValue(ref name, ref lit)) if name == "bound" => {
                        if let Ok(where_predicates) =
                            parse_lit_into_where(cx, name.as_ref(), name.as_ref(), lit) {
                            ser_bound.set(where_predicates.clone());
                            de_bound.set(where_predicates);
                        }
                    }

                    // Parse `#[serde(bound(serialize="D: Serialize", deserialize="D: Deserialize"))]`
                    MetaItem(List(ref name, ref meta_items)) if name == "bound" => {
                        if let Ok((ser, de)) = get_where_predicates(cx, meta_items) {
                            ser_bound.set_opt(ser);
                            de_bound.set_opt(de);
                        }
                    }

                    MetaItem(ref meta_item) => {
                        cx.error(format!("unknown serde field attribute `{}`", meta_item.name()));
                    }

                    Literal(_) => {
                        cx.error("unexpected literal in serde field attribute");
                    }
                }
            }
        }

        // Is skip_deserializing, initialize the field to Default::default()
        // unless a different default is specified by `#[serde(default="...")]`
        if skip_deserializing.0.value.is_some() {
            default.set_if_none(Default::Default);
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
}

type SerAndDe<T> = (Option<T>, Option<T>);

fn get_ser_and_de<T, F>(cx: &Ctxt,
                        attr_name: &'static str,
                        items: &[syn::NestedMetaItem],
                        f: F)
                        -> Result<SerAndDe<T>, ()>
    where F: Fn(&Ctxt, &str, &str, &syn::Lit) -> Result<T, ()>
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
                cx.error(format!("malformed {0} attribute, expected `{0}(serialize = ..., \
                                  deserialize = ...)`",
                                 attr_name));
                return Err(());
            }
        }
    }

    Ok((ser_item.get(), de_item.get()))
}

fn get_renames(cx: &Ctxt, items: &[syn::NestedMetaItem]) -> Result<SerAndDe<String>, ()> {
    get_ser_and_de(cx, "rename", items, get_string_from_lit)
}

fn get_where_predicates(cx: &Ctxt,
                        items: &[syn::NestedMetaItem])
                        -> Result<SerAndDe<Vec<syn::WherePredicate>>, ()> {
    get_ser_and_de(cx, "bound", items, parse_lit_into_where)
}

pub fn get_serde_meta_items(attr: &syn::Attribute) -> Option<Vec<syn::NestedMetaItem>> {
    match attr.value {
        List(ref name, ref items) if name == "serde" => Some(items.iter().cloned().collect()),
        _ => None,
    }
}

fn get_string_from_lit(cx: &Ctxt,
                       attr_name: &str,
                       meta_item_name: &str,
                       lit: &syn::Lit)
                       -> Result<String, ()> {
    if let syn::Lit::Str(ref s, _) = *lit {
        Ok(s.clone())
    } else {
        cx.error(format!("expected serde {} attribute to be a string: `{} = \"...\"`",
                         attr_name,
                         meta_item_name));
        Err(())
    }
}

fn parse_lit_into_path(cx: &Ctxt, attr_name: &str, lit: &syn::Lit) -> Result<syn::Path, ()> {
    let string = try!(get_string_from_lit(cx, attr_name, attr_name, lit));
    syn::parse_path(&string).map_err(|err| cx.error(err))
}

fn parse_lit_into_where(cx: &Ctxt,
                        attr_name: &str,
                        meta_item_name: &str,
                        lit: &syn::Lit)
                        -> Result<Vec<syn::WherePredicate>, ()> {
    let string = try!(get_string_from_lit(cx, attr_name, meta_item_name, lit));
    if string.is_empty() {
        return Ok(Vec::new());
    }

    let where_string = format!("where {}", string);

    syn::parse_where_clause(&where_string).map(|wh| wh.predicates).map_err(|err| cx.error(err))
}
